use crate::vm::Op;

// The goal is to get this to be `Sexpr`
#[derive(Debug, PartialEq, Clone)]
pub enum SimpleExpression {
    Call { op: Op, args: Box<(SimpleExpression, SimpleExpression)> },
    Constant(u8),
    If { condition: Box<SimpleExpression>, then: Box<SimpleExpression>, else_: Box<SimpleExpression> },
}

fn compile_expression(expression: SimpleExpression, code: &mut Vec<u8>) {
    match expression {
        SimpleExpression::Call { op, args } => {
            compile_expression(args.0, code);
            compile_expression(args.1, code);
            code.push(op.into());
        }
        SimpleExpression::Constant(value) => {
            code.push(Op::Load.into());
            code.push(value)
        },
        SimpleExpression::If { condition, then, else_ } => {
            // IF
            compile_expression(*condition, code);

            // skip to "then"
            code.push(Op::CondJump.into());
            code.push(0x00); // will mutate this later
            let place_to_put_then_addr = code.len() - 1;

            // ELSE
            compile_expression(*else_, code);
            
            // skip to end
            code.push(Op::Jump.into());
            // code[to_then_jump_address as usize] = code.len() as u8;
            code.push(0x00); // will mutate this later
            let place_to_put_finish_addr = (code.len() - 1) as u8;

            // THEN
            let then_addr = code.len() as u8;
            code[place_to_put_then_addr] = then_addr;
            compile_expression(*then, code);

            // FINISH
            let finish_addr = code.len() as u8;
            code[place_to_put_finish_addr as usize] = finish_addr
        },
    }
}

fn compile(expression: SimpleExpression) -> Vec<u8> {
    let mut code: Vec<u8> = vec![];
    compile_expression(expression, &mut code);
    code
}

#[cfg(test)]
mod tests {
    use crate::vm::VM;

    use super::*;

    #[test]
    fn test_compile_0() {
        let expression = SimpleExpression::Call {
            op: Op::Add,
            args: Box::new((SimpleExpression::Constant(5), SimpleExpression::Constant(6))),
        };
        let code = compile(expression);
        assert_eq!(code, vec![0x00, 0x05, 0x00, 0x06, 0x01]);

        let mut vm = VM::new();
        vm.load(code);
        vm.run();
        println!("{:?}", vm.stack);
    }

    #[test]
    fn test_compile_compound() {
        let expression = SimpleExpression::Call {
            op: Op::Add,
            args: Box::new((
                SimpleExpression::Call {
                    op: Op::Add,
                    args: Box::new((SimpleExpression::Constant(1), SimpleExpression::Constant(2))),
                },
                SimpleExpression::Call {
                    op: Op::Add,
                    args: Box::new((SimpleExpression::Constant(3), SimpleExpression::Constant(4))),
                },
            )),
        };
        let code = compile(expression);
        assert_eq!(code, vec![
            Op::Load.into(), 1,
            Op::Load.into(), 2,
            Op::Add.into(),
            Op::Load.into(), 3,
            Op::Load.into(), 4,
            Op::Add.into(),
            Op::Add.into(),]);

        let mut vm = VM::new();
        vm.load(code);
        vm.run();
        println!("{:?}", vm.stack);
    }

    #[test]
    fn test_if() {
        let expression = SimpleExpression::If {
            condition: Box::new(SimpleExpression::Constant(1)),
            then: Box::new(SimpleExpression::Constant(2)),
            else_: Box::new(SimpleExpression::Constant(3)),
        };
        let code = compile(expression);
        assert_eq!(code, vec![
            Op::Load.into(),
            1,
            Op::CondJump.into(),
            8, // addr of "load 2" ---.
            Op::Load.into(), //       |
            3, //                     |
            Op::Jump.into(), //       |
            10, // addr of finish --. |
            Op::Load.into(), // <---|-'
            2, //                   |
            // <--------------------'
        ]);
    }

    #[test]
    fn test_if_complex() {
        let expression = SimpleExpression::If {
            condition: Box::new(SimpleExpression::Call {
                op: Op::Add,
                args: Box::new((
                    SimpleExpression::Constant(1),
                    SimpleExpression::Constant(2),
                )),
            }),
            then: Box::new(SimpleExpression::Call {
                op: Op::Add,
                args: Box::new((
                    SimpleExpression::Constant(3),
                    SimpleExpression::Constant(4),
                )),
            }),
            else_: Box::new(SimpleExpression::Call {
                op: Op::Add,
                args: Box::new((
                    SimpleExpression::Constant(5),
                    SimpleExpression::Constant(6),
                )),
            }),
        };
        let code = compile(expression);
        assert_eq!(code, vec![
            Op::Load.into(), 1,
            Op::Load.into(), 2,
            Op::Add.into(),

            Op::CondJump.into(),
            14,

            Op::Load.into(), 5,
            Op::Load.into(), 6,
            Op::Add.into(),

            Op::Jump.into(),
            19,

            /* idx 14 */ 
            Op::Load.into(), 3,
            Op::Load.into(), 4,
            Op::Add.into(),
            // idx 19
        ]);

        // NOTE: This test shouldn't be here but good for easy testing
        // let mut vm = VM::new();
        // vm.load(code);
        // vm.run();
        // assert_eq!(vm.stack, vec![7])
    }
}