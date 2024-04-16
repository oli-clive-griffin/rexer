#![allow(unused, dead_code)]

use crate::vm::{BytecodeChunk, Op, ConstantsValue};

// The goal is to get this to be `Sexpr`
#[derive(Debug, PartialEq, Clone)]
pub enum SimpleExpression {
    Call {
        op: Op,
        args: Box<(SimpleExpression, SimpleExpression)>,
    },
    If {
        condition: Box<SimpleExpression>,
        then: Box<SimpleExpression>,
        else_: Box<SimpleExpression>,
    },
    Constant(ConstantsValue), // TODO don't use this type
}

fn compile_expression(
    expression: SimpleExpression,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantsValue>,
) {
    match expression {
        SimpleExpression::Call { op, args } => {
            compile_expression(args.0, code, constants);
            compile_expression(args.1, code, constants);
            code.push(op.into());
        }
        SimpleExpression::Constant(value) => {
            constants.push(value);
            code.push(Op::Load.into());
            code.push(constants.len() as u8 - 1);
        }
        SimpleExpression::If {
            condition,
            then,
            else_,
        } => {
            // IF
            compile_expression(*condition, code, constants);

            // skip to "then"
            code.push(Op::CondJump.into());
            code.push(0x00); // will mutate this later
            let then_jump_idx = code.len() - 1;

            // ELSE
            compile_expression(*else_, code, constants);

            // skip to end
            code.push(Op::Jump.into());
            // code[to_then_jump_address as usize] = code.len() as u8;
            code.push(0x00); // will mutate this later
            let finish_jump_idx = code.len() - 1;

            // THEN
            let then_jump = (code.len() - then_jump_idx) as u8;
            code[then_jump_idx] = then_jump;
            compile_expression(*then, code, constants);

            // FINISH
            let finish_jump = (code.len() - finish_jump_idx) as u8;
            code[finish_jump_idx as usize] = finish_jump
        }
    }
}

fn compile(expression: SimpleExpression) -> BytecodeChunk {
    let mut code: Vec<u8> = vec![];
    let mut constants: Vec<ConstantsValue> = vec![];
    compile_expression(expression, &mut code, &mut constants);
    BytecodeChunk::new(code, constants)
}

#[cfg(test)]
mod tests {
    use crate::vm::{StackValue, VM};

    use super::*;

    #[test]
    fn test_compile_0() {
        let expression = SimpleExpression::Call {
            op: Op::Add,
            args: Box::new((
                SimpleExpression::Constant(ConstantsValue::Integer(5)),
                SimpleExpression::Constant(ConstantsValue::Integer(6)),
            )),
        };
        let bc = compile(expression);
        assert_eq!(
            bc.code,
            vec![Op::Load.into(), 0, Op::Load.into(), 1, Op::Add.into(),]
        );
        assert_eq!(bc.constants, vec![ConstantsValue::Integer(5), ConstantsValue::Integer(6)]);

        // let mut vm = VM::new();
        // vm.run(bc);
        // println!("{:?}", vm.stack);
    }

    #[test]
    fn test_compile_compound() {
        let expression = SimpleExpression::Call {
            op: Op::Add,
            args: Box::new((
                SimpleExpression::Call {
                    op: Op::Add,
                    args: Box::new((
                        SimpleExpression::Constant(ConstantsValue::Integer(11)),
                        SimpleExpression::Constant(ConstantsValue::Integer(12)),
                    )),
                },
                SimpleExpression::Call {
                    op: Op::Add,
                    args: Box::new((
                        SimpleExpression::Constant(ConstantsValue::Integer(13)),
                        SimpleExpression::Constant(ConstantsValue::Integer(14)),
                    )),
                },
            )),
        };
        let bc = compile(expression);
        assert_eq!(
            bc.code,
            vec![
                Op::Load.into(),
                0,
                Op::Load.into(),
                1,
                Op::Add.into(),
                Op::Load.into(),
                2,
                Op::Load.into(),
                3,
                Op::Add.into(),
                Op::Add.into(),
            ]
        );
        assert_eq!(
            bc.constants,
            vec![
                ConstantsValue::Integer(11),
                ConstantsValue::Integer(12),
                ConstantsValue::Integer(13),
                ConstantsValue::Integer(14),
            ]
        );

        // let mut vm = VM::new();
        // vm.load(code);
        // vm.run();
        // println!("{:?}", vm.stack);
    }

    #[test]
    fn test_if() {
        let expression = SimpleExpression::If {
            condition: Box::new(SimpleExpression::Constant(ConstantsValue::Integer(11))),
            then: Box::new(SimpleExpression::Constant(ConstantsValue::Integer(12))),
            else_: Box::new(SimpleExpression::Constant(ConstantsValue::Integer(13))),
        };
        let bc = compile(expression);
        assert_eq!(
            bc.code,
            vec![
                Op::Load.into(),
                0,
                Op::CondJump.into(),
                5,
                Op::Load.into(),
                1,
                Op::Jump.into(),
                3,
                Op::Load.into(),
                2,
            ]
        );
        assert_eq!(
            bc.constants,
            vec![ConstantsValue::Integer(11), ConstantsValue::Integer(13), ConstantsValue::Integer(12)]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::new();
        vm.run(bc);
        assert_eq!(vm.stack, vec![StackValue::Integer(12)])
    }

    #[test]
    fn test_if_complex() {
        let expression = SimpleExpression::If {
            condition: Box::new(SimpleExpression::Call {
                op: Op::Add,
                args: Box::new((
                    SimpleExpression::Constant(ConstantsValue::Integer(11)),
                    SimpleExpression::Constant(ConstantsValue::Integer(12)),
                )),
            }),
            then: Box::new(SimpleExpression::Call {
                op: Op::Add,
                args: Box::new((
                    SimpleExpression::Constant(ConstantsValue::Integer(13)),
                    SimpleExpression::Constant(ConstantsValue::Integer(14)),
                )),
            }),
            else_: Box::new(SimpleExpression::Call {
                op: Op::Add,
                args: Box::new((
                    SimpleExpression::Constant(ConstantsValue::Integer(15)),
                    SimpleExpression::Constant(ConstantsValue::Integer(16)),
                )),
            }),
        };
        let bc = compile(expression);
        assert_eq!(
            bc.code,
            vec![
                Op::Load.into(),
                0,
                Op::Load.into(),
                1,
                Op::Add.into(),
                Op::CondJump.into(),
                8,
                Op::Load.into(),
                2,
                Op::Load.into(),
                3,
                Op::Add.into(),
                Op::Jump.into(),
                6,
                Op::Load.into(),
                4,
                Op::Load.into(),
                5,
                Op::Add.into(),
                // idx 19
            ]
        );
        assert_eq!(
            bc.constants,
            vec![
                ConstantsValue::Integer(11),
                ConstantsValue::Integer(12),
                ConstantsValue::Integer(15),
                ConstantsValue::Integer(16),
                ConstantsValue::Integer(13),
                ConstantsValue::Integer(14),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::new();
        vm.run(bc);
        assert_eq!(vm.stack, vec![StackValue::Integer(27)])
    }
}
