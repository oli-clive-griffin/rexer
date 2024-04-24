#![allow(unused, dead_code)]

use std::collections::HashMap;

use crate::{sexpr::Sexpr, vm::{BytecodeChunk, ConstantValue, Function, ObjectValue, Op}};

// The goal is to get this to be `Sexpr`
#[derive(Debug, PartialEq, Clone)]
pub enum SimpleExpression {
    Quote(Sexpr),
    RegularForm(Vec<SimpleExpression>),
    If {
        condition: Box<SimpleExpression>,
        then: Box<SimpleExpression>,
        else_: Box<SimpleExpression>,
    },
    Constant(ConstantValue), // TODO maybe don't use this ConstantValue type here, make own one.
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
    constants: &mut Vec<ConstantValue>,
) {
    match expression {
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
            constants.push(ConstantValue::Object(ObjectValue::String(name)));
            code.push(constants.len() as u8 - 1);
        }
        SimpleExpression::Symbol(symbol) => {
            code.push(Op::ReferenceGlobal.into());
            constants.push(ConstantValue::Object(ObjectValue::String(symbol)));
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
            let arity = {
                let arity = exprs.len() - 1;
                if arity > 255 {
                    panic!()
                }
                arity as u8
            };

            for expr in exprs {
                compile_expression(expr, code, constants);
            }

            code.push(Op::FuncCall.into());
            code.push(arity);
        }
        SimpleExpression::Quote(sexpr) => match sexpr {
            Sexpr::List { quasiquote, sexprs } => {
                if quasiquote {
                    todo!("quasiquote not implemented")
                }
                // nil for end of list
                code.push(Op::Constant.into());
                constants.push(ConstantValue::Nil);
                code.push(constants.len() as u8 - 1);

                // cons each element in reverse order
                for expr in sexprs.iter().rev() {
                    compile_expression(
                        SimpleExpression::Quote(expr.clone()), //
                        code,
                        constants,
                    );
                    code.push(Op::Cons.into())
                }
            }
            Sexpr::Symbol(s) => {
                code.push(Op::Constant.into());
                constants.push(ConstantValue::Object(ObjectValue::Symbol(s)));
                code.push(constants.len() as u8 - 1);
            }
            Sexpr::Int(i) => {
                code.push(Op::Constant.into());
                constants.push(ConstantValue::Integer(i));
                code.push(constants.len() as u8 - 1);
            }
            Sexpr::Float(f) => {
                code.push(Op::Constant.into());
                constants.push(ConstantValue::Float(f));
                code.push(constants.len() as u8 - 1);
            }
            Sexpr::String(_) => todo!(),
            Sexpr::Bool(_) => todo!(),
            Sexpr::Function { parameters, body } => todo!(),
            Sexpr::Macro { parameters, body } => todo!(),
            Sexpr::BuiltIn(_) => todo!(),
            Sexpr::CommaUnquote(_) => todo!(),
            Sexpr::Nil => todo!(),
        },
    }
}

pub fn compile_program(expressions: Vec<SimpleExpression>) -> BytecodeChunk {
    let mut code: Vec<u8> = vec![];
    let mut constants: Vec<ConstantValue> = vec![];
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
    fn test_if() {
        let expression = SimpleExpression::If {
            condition: Box::new(SimpleExpression::Constant(ConstantValue::Integer(11))),
            then: Box::new(SimpleExpression::Constant(ConstantValue::Integer(12))),
            else_: Box::new(SimpleExpression::Constant(ConstantValue::Integer(13))),
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
                ConstantValue::Integer(11),
                ConstantValue::Integer(13),
                ConstantValue::Integer(12)
            ]
        );

        // NOTE: This test shouldn't be here but good for easy testing
        let mut vm = VM::default();
        vm.run(bc);
        assert_eq!(vm.stack, StaticStack::from([SmallValue::Integer(12)]))
    }

    #[test]
    fn test_declare_global() {
        let expression = SimpleExpression::DeclareGlobal {
            name: "foo".to_string(),
            value: Box::new(SimpleExpression::Constant(ConstantValue::Integer(11))),
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
                ConstantValue::Integer(11),
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
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
                value: Box::new(SimpleExpression::RegularForm(vec![
                    SimpleExpression::Symbol("+".to_string()),
                    SimpleExpression::Constant(ConstantValue::Integer(11)),
                    SimpleExpression::Constant(ConstantValue::Integer(12)),
                ])),
            },
            SimpleExpression::Symbol("foo".to_string()),
        ];

        let bc = compile_program(program);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
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
            SimpleExpression::Constant(ConstantValue::Integer(11)),
            SimpleExpression::Constant(ConstantValue::Integer(12)),
        ])]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ObjectValue::String("*".to_string())),
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
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
                SimpleExpression::Constant(ConstantValue::Integer(11)),
                SimpleExpression::Constant(ConstantValue::Integer(12)),
            ]),
            SimpleExpression::RegularForm(vec![
                SimpleExpression::Symbol("+".to_string()),
                SimpleExpression::Constant(ConstantValue::Integer(13)),
                SimpleExpression::Constant(ConstantValue::Integer(14)),
            ]),
        ])]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ObjectValue::String("+".to_string())),
                ConstantValue::Object(ObjectValue::String("+".to_string())),
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
                ConstantValue::Integer(13),
                ConstantValue::Integer(14),
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

    #[test]
    fn test_cons() {
        let bc = compile_program(vec![SimpleExpression::Quote(Sexpr::List {
            quasiquote: false,
            sexprs: vec![
                Sexpr::Int(1),
                Sexpr::Int(2),
                Sexpr::Int(3),
            ],
        })]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Nil,
                ConstantValue::Integer(3),
                ConstantValue::Integer(2),
                ConstantValue::Integer(1),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0, // load nil
                Op::Constant.into(),
                1, // load 3
                Op::Cons.into(),
                Op::Constant.into(),
                2, // load 2
                Op::Cons.into(),
                Op::Constant.into(),
                3, // load 1
                Op::Cons.into(),
                Op::DebugEnd.into(),
            ]
        );
    }

    #[test]
    fn test_cons_nested() {
        let bc = compile_program(vec![SimpleExpression::Quote(Sexpr::List {
            quasiquote: false,
            sexprs: vec![
                Sexpr::Int(10),
                Sexpr::List {
                    quasiquote: false,
                    sexprs: vec![Sexpr::Int(20), Sexpr::Int(30)],
                },
                Sexpr::Int(40),
            ],
        })]);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Nil,
                ConstantValue::Integer(40),
                ConstantValue::Nil,
                ConstantValue::Integer(30),
                ConstantValue::Integer(20),
                ConstantValue::Integer(10),
            ]
        );
        
        assert_eq!(
            bc.code,
            vec![
                Op::Constant.into(),
                0, // load nil
                Op::Constant.into(),
                1, // load 4
                Op::Cons.into(),
                Op::Constant.into(),
                2, // load nil
                Op::Constant.into(),
                3, // load 3
                Op::Cons.into(),
                Op::Constant.into(),
                4, // load 2
                Op::Cons.into(),
                Op::Cons.into(),
                Op::Constant.into(),
                5, // load 1
                Op::Cons.into(),
                Op::DebugEnd.into(),
            ]
        );

        // let mut vm = VM::default();
        // vm.run(bc);
        // let list = *vm.stack.at(0).unwrap();
        // match list {
        //     SmallValue::Object(o) => println!("{}", unsafe { &*o }.value),
        //     _ => panic!("Expected list"),
        // }
    }
}
