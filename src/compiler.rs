#![allow(unused, dead_code)]

use std::collections::HashMap;

use crate::vm::{BytecodeChunk, ConstantsValue, Function, ObjectValue, Op};

// The goal is to get this to be `Sexpr`
#[derive(Debug, PartialEq, Clone)]
pub enum SimpleExpression {
    Add(Box<(SimpleExpression, SimpleExpression)>),
    Mul(Box<(SimpleExpression, SimpleExpression)>),
    Sub(Box<(SimpleExpression, SimpleExpression)>),
    Div(Box<(SimpleExpression, SimpleExpression)>),
    Quote(Box<SimpleExpression>),
    // RegularForm {
    //     car: Box<SimpleExpression>,
    //     cdr: Vec<SimpleExpression>,
    // },
    RegularForm(Vec<SimpleExpression>),
    If {
        condition: Box<SimpleExpression>,
        then: Box<SimpleExpression>,
        else_: Box<SimpleExpression>,
    },
    Constant(ConstantsValue), // TODO don't use this type
    DeclareGlobal {
        name: String,
        value: Box<SimpleExpression>,
    },
    Symbol(String),
    DebugPrint(Box<SimpleExpression>),
}

fn compile_expression(
    expression: SimpleExpression,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantsValue>,
) {
    match expression {
        SimpleExpression::Add(args) => {
            compile_expression((*args).0, code, constants);
            compile_expression((*args).1, code, constants);
            code.push(Op::Add.into());
        }
        SimpleExpression::Sub(args) => {
            compile_expression((*args).0, code, constants);
            compile_expression((*args).1, code, constants);
            code.push(Op::Sub.into());
        }
        SimpleExpression::Mul(args) => {
            compile_expression((*args).0, code, constants);
            compile_expression((*args).1, code, constants);
            code.push(Op::Mul.into());
        }
        SimpleExpression::Div(args) => {
            compile_expression((*args).0, code, constants);
            compile_expression((*args).1, code, constants);
            code.push(Op::Div.into());
        }
        SimpleExpression::Constant(value) => {
            constants.push(value);
            code.push(Op::Constant.into());
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
            code[finish_jump_idx] = finish_jump
        }
        SimpleExpression::DeclareGlobal { name, value } => {
            compile_expression(*value, code, constants);
            code.push(Op::DeclareGlobal.into());
            constants.push(ConstantsValue::Object(ObjectValue::String(name)));
            code.push(constants.len() as u8 - 1);
        }
        SimpleExpression::Symbol(symbol) => {
            code.push(Op::ReferenceGlobal.into());
            constants.push(ConstantsValue::Object(ObjectValue::String(symbol)));
            code.push(constants.len() as u8 - 1);

            // let constant_idx = constants
            //     .iter()
            //     .position(|x| match x {
            //         ConstantsValue::Object(ObjectValue::String(s)) => s == &symbol,
            //         _ => false,
            //     });

            // if let Some(idx) = constant_idx {
            //     code.push(Op::ReferenceGlobal.into());
            //     code.push(idx as u8);
            //     return;
            // }
            // todo!("local variable lookup not implemented")
        }
        SimpleExpression::DebugPrint(expr) => {
            compile_expression(*expr, code, constants);
            code.push(Op::DebugPrint.into());
        }
        SimpleExpression::RegularForm(exprs) => {
            // We don't know the arity of the function at compile-time so we
            // defensively put the number of arguments to check at runtime
            let arity = exprs.len() - 1;
            let arity = if arity > 255 { panic!() } else { arity as u8 };

            for expr in exprs {
                compile_expression(expr, code, constants);
            }

            code.push(Op::FuncCall.into());
            code.push(arity);
        }
        SimpleExpression::Quote(_) => todo!(), // SimpleExpression::Quote(sexpr) => {
          //     match *sexpr {
          //         SimpleExpression::RegularForm(exprs) => {
          //             for expr in exprs.iter().rev() {
          //                 match
          //                 constants.push(expr)
          //             }
          //         }
          //         _ => todo!("quote not implemented for {:?}", sexpr),
          //     }
          // }
          // 
    }
}

pub fn compile_program(expressions: Vec<SimpleExpression>) -> BytecodeChunk {
    let mut code: Vec<u8> = vec![];
    let mut constants: Vec<ConstantsValue> = vec![];
    for expression in expressions {
        compile_expression(expression, &mut code, &mut constants);
    }
    code.push(Op::DebugEnd.into());
    BytecodeChunk::new(code, constants)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        static_stack::StaticStack,
        vm::{SmallValue, VM},
    };

    use super::*;

    #[test]
    fn test_compile_0() {
        let expression = SimpleExpression::Add(Box::new((
            SimpleExpression::Constant(ConstantsValue::Integer(5)),
            SimpleExpression::Constant(ConstantsValue::Integer(6)),
        )));
        let bc = compile_program(vec![expression]);
        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0,
                Op::Constant.into(),
                1,
                Op::Add.into(),
                Op::DebugEnd.into(),
            ]
        );
        assert_eq!(
            bc.constants,
            vec![ConstantsValue::Integer(5), ConstantsValue::Integer(6)]
        );
    }

    #[test]
    fn test_compile_compound() {
        let expression = SimpleExpression::Add(Box::new((
            SimpleExpression::Add(Box::new((
                SimpleExpression::Constant(ConstantsValue::Integer(11)),
                SimpleExpression::Constant(ConstantsValue::Integer(12)),
            ))),
            SimpleExpression::Add(Box::new((
                SimpleExpression::Constant(ConstantsValue::Integer(13)),
                SimpleExpression::Constant(ConstantsValue::Integer(14)),
            ))),
        )));
        let bc = compile_program(vec![expression]);
        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0,
                Op::Constant.into(),
                1,
                Op::Add.into(),
                Op::Constant.into(),
                2,
                Op::Constant.into(),
                3,
                Op::Add.into(),
                Op::Add.into(),
                Op::DebugEnd.into(),
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
    }

    #[test]
    fn test_if() {
        let expression = SimpleExpression::If {
            condition: Box::new(SimpleExpression::Constant(ConstantsValue::Integer(11))),
            then: Box::new(SimpleExpression::Constant(ConstantsValue::Integer(12))),
            else_: Box::new(SimpleExpression::Constant(ConstantsValue::Integer(13))),
        };
        let bc = compile_program(vec![expression]);
        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0,
                Op::CondJump.into(),
                5,
                Op::Constant.into(),
                1,
                Op::Jump.into(),
                3,
                Op::Constant.into(),
                2,
                Op::DebugEnd.into(),
            ]
        );
        assert_eq!(
            bc.constants,
            vec![
                ConstantsValue::Integer(11),
                ConstantsValue::Integer(13),
                ConstantsValue::Integer(12)
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(12)]))
    }

    #[test]
    fn test_if_complex() {
        let expression = SimpleExpression::If {
            condition: Box::new(SimpleExpression::Add(Box::new((
                SimpleExpression::Constant(ConstantsValue::Integer(11)),
                SimpleExpression::Constant(ConstantsValue::Integer(12)),
            )))),
            then: Box::new(SimpleExpression::Add(Box::new((
                SimpleExpression::Constant(ConstantsValue::Integer(13)),
                SimpleExpression::Constant(ConstantsValue::Integer(14)),
            )))),
            else_: Box::new(SimpleExpression::Add(Box::new((
                SimpleExpression::Constant(ConstantsValue::Integer(15)),
                SimpleExpression::Constant(ConstantsValue::Integer(16)),
            )))),
        };
        let bc = compile_program(vec![expression]);
        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0,
                Op::Constant.into(),
                1,
                Op::Add.into(),
                Op::CondJump.into(),
                8,
                Op::Constant.into(),
                2,
                Op::Constant.into(),
                3,
                Op::Add.into(),
                Op::Jump.into(),
                6,
                Op::Constant.into(),
                4,
                Op::Constant.into(),
                5,
                Op::Add.into(),
                // idx 19
                Op::DebugEnd.into(),
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
    }

    #[test]
    fn test_declare_global() {
        let expression = SimpleExpression::DeclareGlobal {
            name: "foo".to_string(),
            value: Box::new(SimpleExpression::Constant(ConstantsValue::Integer(11))),
        };
        let bc = compile_program(vec![expression]);
        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0,
                Op::DeclareGlobal.into(),
                1,
                Op::DebugEnd.into(),
            ]
        );
        assert_eq!(
            bc.constants,
            vec![
                ConstantsValue::Integer(11),
                ConstantsValue::Object(ObjectValue::String("foo".to_string())),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.globals.get("foo"), Some(&SmallValue::Integer(11)))
    }

    #[test]
    fn test_assign_global_as_expr() {
        let program = vec![
            SimpleExpression::DeclareGlobal {
                name: "foo".to_string(),
                value: Box::new(SimpleExpression::Add(Box::new((
                    SimpleExpression::Constant(ConstantsValue::Integer(11)),
                    SimpleExpression::Constant(ConstantsValue::Integer(12)),
                )))),
            },
            SimpleExpression::Symbol("foo".to_string()),
        ];

        let bc = compile_program(program);

        assert_eq!(
            bc.constants,
            vec![
                ConstantsValue::Integer(11),
                ConstantsValue::Integer(12),
                ConstantsValue::Object(ObjectValue::String("foo".to_string())),
                ConstantsValue::Object(ObjectValue::String("foo".to_string())),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0,
                Op::Constant.into(),
                1,
                Op::Add.into(),
                Op::DeclareGlobal.into(),
                2,
                Op::ReferenceGlobal.into(),
                3,
                Op::DebugEnd.into(),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.globals.get("foo"), Some(&SmallValue::Integer(23)));
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(23)]))
    }

    #[test]
    fn test_call_function() {
        let bc = compile_program(vec![SimpleExpression::RegularForm(vec![
            SimpleExpression::Symbol("*".to_string()).into(),
            SimpleExpression::Constant(ConstantsValue::Integer(11)),
            SimpleExpression::Constant(ConstantsValue::Integer(12)),
        ])]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantsValue::Object(ObjectValue::String("*".to_string())),
                ConstantsValue::Integer(11),
                ConstantsValue::Integer(12),
            ],
        );

        assert_eq!(
            bc.code,
            vec![
                Op::ReferenceGlobal.into(),
                0, // load function symbol
                Op::Constant.into(),
                1, // load arg 1
                Op::Constant.into(),
                2, // load arg 2
                Op::FuncCall.into(),
                2, // call function with arity 2
                Op::DebugEnd.into(),
            ]
        );
    }

    #[test]
    fn test_function_with_computed_arguments() {
        let bc = compile_program(vec![SimpleExpression::RegularForm(vec![
            SimpleExpression::Symbol("+".to_string()),
            SimpleExpression::RegularForm(vec![
                SimpleExpression::Symbol("+".to_string()),
                SimpleExpression::Constant(ConstantsValue::Integer(11)),
                SimpleExpression::Constant(ConstantsValue::Integer(12)),
            ]),
            SimpleExpression::Mul(Box::new((
                SimpleExpression::Constant(ConstantsValue::Integer(13)),
                SimpleExpression::Constant(ConstantsValue::Integer(14)),
            ))),
        ])]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantsValue::Object(ObjectValue::String("+".to_string())),
                ConstantsValue::Object(ObjectValue::String("+".to_string())),
                ConstantsValue::Integer(11),
                ConstantsValue::Integer(12),
                ConstantsValue::Integer(13),
                ConstantsValue::Integer(14),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::ReferenceGlobal.into(),
                0, // reference function symbol (outer)
                //
                // arg 1 (outer)
                Op::ReferenceGlobal.into(),
                1, // reference function symbol (inner)
                // arg 1 (inner)
                Op::Constant.into(),
                2, // load arg 1 "11"
                // arg 2 (inner)
                Op::Constant.into(),
                3, // load arg 2 "12"
                Op::FuncCall.into(),
                2, // call inner function with arity 2
                //
                // arg 2 (outer)
                Op::Constant.into(),
                4, // load arg to Mul "13"
                Op::Constant.into(),
                5, // load arg to Mul "14"
                Op::Mul.into(),
                Op::FuncCall.into(),
                2, // call function "add" (outer) with arity 2
                Op::DebugEnd.into(),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(205)]));
    }
}
