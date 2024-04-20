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
    FunctionCall {
        name: String,
        args: Vec<SimpleExpression>,
    },
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
            let constant_idx = constants
                .iter()
                .position(|x| match x {
                    ConstantsValue::Object(ObjectValue::String(s)) => s == &symbol,
                    _ => false,
                })
                .expect("Symbol not found in constants");
            code.push(constant_idx as u8);
        }
        SimpleExpression::DebugPrint(expr) => {
            compile_expression(*expr, code, constants);
            code.push(Op::DebugPrint.into());
        }
        SimpleExpression::FunctionCall { name, args } => {
            // todo this is shit but fine for now
            let function_constants_idx = constants
                .iter()
                .position(|x| match x {
                    ConstantsValue::Object(ObjectValue::Function(Function { name: n, .. })) => {
                        n == &name
                    }
                    _ => false,
                })
                .expect("Symbol not found in constants");
            code.push(Op::Constant.into());
            code.push(function_constants_idx as u8);

            let arity = args.len();
            let arity = if arity > 255 { panic!() } else { arity as u8 };

            for arg in args {
                compile_expression(arg, code, constants);
            }

            // finally, put the func-call op code, whose operand is the arity
            code.push(Op::FuncCall.into());
            code.push(arity)
        }
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
        vm::{StackValue, VM},
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
        assert_eq!(vm.stack, StaticStack::from(vec![StackValue::Integer(12)]))
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
        assert_eq!(vm.globals.get("foo"), Some(&StackValue::Integer(11)))
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
                2,
                Op::DebugEnd.into(),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.globals.get("foo"), Some(&StackValue::Integer(23)));
        assert_eq!(vm.stack, StaticStack::from(vec![StackValue::Integer(23)]))
    }

    #[test]
    fn test_call_function() {
        let mut code: Vec<u8> = vec![];
        let mut constants: Vec<ConstantsValue> = vec![ConstantsValue::Object(
            ObjectValue::Function(Function::new(
                "add".to_string(),
                2,
                BytecodeChunk::new(
                    vec![
                        Op::ReferenceLocal.into(),
                        1,
                        Op::ReferenceLocal.into(),
                        2,
                        Op::Add.into(),
                        Op::Return.into(),
                    ],
                    vec![],
                ),
            )),
        )];
        compile_expression(
            SimpleExpression::FunctionCall {
                name: "add".to_string(),
                args: vec![
                    SimpleExpression::Constant(ConstantsValue::Integer(11)),
                    SimpleExpression::Constant(ConstantsValue::Integer(12)),
                ],
            },
            &mut code,
            &mut constants,
        );
        code.push(Op::DebugEnd.into());

        assert_eq!(
            code,
            vec![
                Op::Constant.into(),
                0, // load function
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
        let mut code: Vec<u8> = vec![];
        let mut constants: Vec<ConstantsValue> = vec![ConstantsValue::Object(
            ObjectValue::Function(Function::new(
                "add".to_string(),
                2,
                BytecodeChunk::new(
                    vec![
                        Op::ReferenceLocal.into(),
                        1,
                        Op::ReferenceLocal.into(),
                        2,
                        Op::Add.into(),
                        Op::Return.into(),
                    ],
                    vec![],
                ),
            )),
        )];
        compile_expression(
            SimpleExpression::FunctionCall {
                name: "add".to_string(),
                args: vec![
                    SimpleExpression::FunctionCall {
                        name: "add".to_string(),
                        args: vec![
                            SimpleExpression::Constant(ConstantsValue::Integer(11)),
                            SimpleExpression::Constant(ConstantsValue::Integer(12)),
                        ],
                    },
                    SimpleExpression::Mul(Box::new((
                        SimpleExpression::Constant(ConstantsValue::Integer(13)),
                        SimpleExpression::Constant(ConstantsValue::Integer(14)),
                    ))),
                ],
            },
            &mut code,
            &mut constants,
        );

        code.push(Op::DebugEnd.into());

        assert_eq!(
            constants[1..],
            vec![
                ConstantsValue::Integer(11),
                ConstantsValue::Integer(12),
                ConstantsValue::Integer(13),
                ConstantsValue::Integer(14),
            ]
        );

        assert_eq!(
            code,
            vec![
                Op::Constant.into(),
                0, // load function (outer)
                Op::Constant.into(),
                0, // load function (inner)
                // args for inner function
                Op::Constant.into(),
                1, // load arg 1 "11"
                Op::Constant.into(),
                2, // load arg 2 "12"
                Op::FuncCall.into(),
                2, // call inner function with arity 2
                // next arg for outer function
                Op::Constant.into(),
                3, // load arg to Mul "13"
                Op::Constant.into(),
                4, // load arg to Mul "14"
                Op::Mul.into(),
                Op::FuncCall.into(),
                2, // call function "add" (outer) with arity 2
                Op::DebugEnd.into(),
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(BytecodeChunk { code, constants });
        assert_eq!(vm.stack, StaticStack::from(vec![StackValue::Integer(205)]));
    }
}
