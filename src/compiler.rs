use crate::{
    sexpr::Sexpr,
    vm::{BytecodeChunk, ConstantValue, Function, ObjectValue, Op},
};

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
    GlobalFunctionDeclaration(Box<GlobalFunctionDeclaration>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct GlobalFunctionDeclaration {
    name: String,
    parameters: Vec<String>,
    body: Vec<SimpleExpression>,
}

fn compile_expression(
    expression: SimpleExpression,
    code: &mut Vec<u8>,
    constants: &mut Vec<ConstantValue>,
    locals: &mut Vec<String>,
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
            compile_expression(*condition, code, constants, locals);

            // skip to "then"
            code.push(Op::CondJump.into());
            code.push(0x00); // will mutate this later
            let then_jump_idx = code.len() - 1;

            // ELSE
            compile_expression(*else_, code, constants, locals);

            // skip to end
            code.push(Op::Jump.into());
            // code[to_then_jump_address as usize] = code.len() as u8;
            code.push(0x00); // will mutate this later
            let finish_jump_idx = code.len() - 1;

            // THEN
            let then_jump = (code.len() - then_jump_idx) as u8;
            code[then_jump_idx] = then_jump;
            compile_expression(*then, code, constants, locals);

            // FINISH
            let finish_jump = (code.len() - finish_jump_idx) as u8;
            code[finish_jump_idx] = finish_jump
        }
        SimpleExpression::DeclareGlobal { name, value } => {
            compile_expression(*value, code, constants, locals);
            code.push(Op::DeclareGlobal.into());
            constants.push(ConstantValue::Object(ObjectValue::String(name)));
            code.push(constants.len() as u8 - 1);
        }
        SimpleExpression::Symbol(symbol) => {
            // local / function argument
            let local_idx = locals.iter().position(|x| x == &symbol);
            if let Some(idx) = local_idx {
                code.push(Op::ReferenceLocal.into());
                code.push((idx + 1) as u8); // plus 1 because locals are 1-indexed
                return;
            }

            // fall back to global
            code.push(Op::ReferenceGlobal.into());
            // this can be optimized by reusing the same constant for the same symbol
            // also - this is one of those wierd/cool cases where a language concept becomes a runtime concept: the symbol in the code is a runtime value
            constants.push(ConstantValue::Object(ObjectValue::String(symbol)));
            code.push(constants.len() as u8 - 1);
        }
        SimpleExpression::DebugPrint(expr) => {
            compile_expression(*expr, code, constants, locals);
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
                compile_expression(expr, code, constants, locals);
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
                        locals,
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
            Sexpr::Function { parameters: _, body: _ } => todo!(),
            Sexpr::Macro { parameters: _, body: _ } => todo!(),
            Sexpr::BuiltIn(_) => todo!(),
            Sexpr::CommaUnquote(_) => todo!(),
            Sexpr::Nil => todo!(),
        },
        SimpleExpression::GlobalFunctionDeclaration(func_dec) => {
            // jank way of doing it for now:
            
            // make function object
            let name = func_dec.name.clone();
            let function_obj = ConstantValue::Object(ObjectValue::Function(compile_function(*func_dec)));

            // reference constant function object on stack
            code.push(Op::Constant.into());
            constants.push(function_obj);
            code.push(constants.len() as u8 - 1);

            // declare top of stack as global
            code.push(Op::DeclareGlobal.into());
            constants.push(ConstantValue::Object(ObjectValue::String(name)));
            code.push(constants.len() as u8 - 1);
        }
    }
}

fn compile_function(declaration: GlobalFunctionDeclaration) -> Function {
    let mut code = vec![];
    let mut constants = vec![];
    let mut locals = declaration.parameters.clone();

    for expr in declaration.body {
        compile_expression(expr, &mut code, &mut constants, &mut locals);
    }
    
    code.push(Op::Return.into());

    Function {
        name: declaration.name,
        arity: declaration.parameters.len(),
        bytecode: Box::new(BytecodeChunk::new(code, constants)),
    }
}

pub fn compile_sexprs(sexprs: Vec<Sexpr>) -> BytecodeChunk {
    // println!("sexprs: {:#?}", sexprs);
    let expressions = sexprs.iter().map(map).collect::<Vec<SimpleExpression>>();
    // println!("expressions: {:#?}", expressions);
    compile_expressions(expressions)
}

pub fn compile_expressions(expressions: Vec<SimpleExpression>) -> BytecodeChunk {
    let mut code: Vec<u8> = vec![];
    let mut constants: Vec<ConstantValue> = vec![];
    let mut locals: Vec<String> = vec![];
    for expression in expressions {
        compile_expression(expression, &mut code, &mut constants, &mut locals);
    }
    code.push(Op::DebugEnd.into());
    BytecodeChunk::new(code, constants)
}

/// DRAFT
pub fn map(sexpr: &Sexpr) -> SimpleExpression {
    match sexpr {
        Sexpr::Symbol(sym) => SimpleExpression::Symbol(sym.clone()),
        Sexpr::String(str) => {
            SimpleExpression::Constant(ConstantValue::Object(ObjectValue::String(str.clone())))
        }
        Sexpr::Bool(bool) => SimpleExpression::Constant(ConstantValue::Boolean(*bool)),
        Sexpr::Int(i) => SimpleExpression::Constant(ConstantValue::Integer(*i)),
        Sexpr::Float(f) => SimpleExpression::Constant(ConstantValue::Float(*f)),
        Sexpr::Function {
            parameters: _,
            body: _,
        } => {
            panic!("raw function node should not be present in this context")
        }
        Sexpr::Macro {
            parameters: _,
            body: _,
        } => {
            panic!("raw macro node should not be present in this context")
        }
        Sexpr::BuiltIn(_) => {
            todo!();
        }
        Sexpr::CommaUnquote(_) => {
            todo!("unquote not implemented")
        }
        Sexpr::Nil => SimpleExpression::Constant(ConstantValue::Nil),
        Sexpr::List { quasiquote, sexprs } => {
            if *quasiquote {
                todo!("quasiquote not implemented")
            }

            if sexprs.is_empty() {
                panic!("empty unquoted list")
            }
            if let Some(special_form) = map_to_special_form(sexprs) {
                return special_form;
            }
            SimpleExpression::RegularForm(sexprs.iter().map(map).collect())
        }
    }
}

fn map_to_special_form(sexprs: &[Sexpr]) -> Option<SimpleExpression> {
    let head = sexprs.first().unwrap();

    if let Sexpr::Symbol(sym) = head {
        match sym.as_str() {
            "if" => {
                return Some(SimpleExpression::If {
                    condition: Box::new(map(&sexprs[1])),
                    then: Box::new(map(&sexprs[2])),
                    else_: Box::new(map(&sexprs[3])),
                });
            }
            "set!" => {
                let name = match &sexprs[1] {
                    Sexpr::Symbol(s) => s,
                    _ => panic!("set! expects symbol as first argument"),
                };
                return Some(SimpleExpression::DeclareGlobal {
                    name: name.to_string(),
                    value: Box::new(map(&sexprs[2])),
                });
            }
            "quote" => {
                if sexprs.len() != 2 {
                    panic!("quote expects 1 argument")
                }
                return Some(SimpleExpression::Quote(sexprs[1].clone()));
            }
            "debug-print" => {
                if sexprs.len() != 2 {
                    panic!("debug-print expects 1 argument")
                }
                return Some(SimpleExpression::DebugPrint(Box::new(map(&sexprs[1]))));
            }
            "fn" => {
                let (name, parameters) = match &sexprs[1] {
                    Sexpr::List { quasiquote, sexprs } => {
                        if *quasiquote {
                            todo!("inappropriate quasiquote")
                        }
                        let name = match &sexprs[0] {
                            Sexpr::Symbol(s) => s.clone(),
                            _ => panic!("expected symbol for function name"),
                        };
                        let parameters = sexprs[1..]
                            .iter()
                            .map(|sexpr| match sexpr {
                                Sexpr::Symbol(s) => s.clone(),
                                _ => panic!("expected symbol for parameter"),
                            })
                            .collect();
                        (name, parameters)
                    }
                    got => panic!(
                        "expected list for function signature declaration, got {:?}",
                        got
                    ),
                };

                let body = sexprs[2..].iter().map(map).collect();

                return Some(SimpleExpression::GlobalFunctionDeclaration(Box::new(
                    GlobalFunctionDeclaration {
                        parameters,
                        body,
                        name,
                    },
                )));
            }
            _ => {
                println!("head: {:?} was not a special form", head);
            }
        };
    }
    None
}

pub fn disassemble(bc: &BytecodeChunk) -> String {
    let mut pc = 0;
    let mut lines = "".to_string();
    while pc < bc.code.len() {
        let op: Op = bc.code[pc].try_into().expect("invalid opcode");
        let line: String = match op {
            Op::Add => "Add".to_string(),
            Op::Sub => "Sub".to_string(),
            Op::Mul => "Mul".to_string(),
            Op::Div => "Div".to_string(),
            Op::Neg => "Neg".to_string(),
            Op::Return => "Return".to_string(),
            Op::Cons => "Cons".to_string(),
            Op::DebugEnd => "DebugEnd".to_string(),
            Op::DebugPrint => "DebugPrint".to_string(),
            Op::Constant => {
                pc += 1;
                let idx = bc.code[pc];
                // format!("Constant idx: {idx} val: {:?}", bc.constants[idx as usize])
                format!("Constant\n  val: {:?}", bc.constants[idx as usize])
            }
            Op::Jump => {
                pc += 1;
                let offset = bc.code[pc];
                format!("Jump\n  offset: {offset}")
            }
            Op::CondJump => {
                pc += 1;
                let offset = bc.code[pc];
                format!("CondJump\n  offset: {offset}")
            }
            Op::FuncCall => {
                pc += 1;
                let arity = bc.code[pc];
                format!("FuncCall\n  arity: {arity}")
            }
            Op::DeclareGlobal => {
                pc += 1;
                let name_idx = bc.code[pc];
                let name = match &bc.constants[name_idx as usize] {
                    ConstantValue::Object(o) => match o {
                        ObjectValue::String(s) => s,
                        got => panic!("expected string for global name, got {:?}", got),
                    }
                    got => panic!("expected object for global name, got {:?}", got),
                };

                // value is on the stack
                format!("DeclareGlobal\n  name: {name} (value on stack)")
            }
            Op::ReferenceGlobal => {
                pc += 1;
                let name_idx = bc.code[pc];
                let name = match &bc.constants[name_idx as usize] {
                    ConstantValue::Object(o) => match o {
                        ObjectValue::String(s) => s,
                        got => panic!("expected string for global name, got {:?}", got),
                    }
                    got => panic!("expected object for global name, got {:?}", got),
                };
                
                format!("ReferenceGlobal\n  name: {name}")
            }
            Op::ReferenceLocal => {
                pc += 1;
                let idx = bc.code[pc];
                format!("ReferenceLocal\n  idx: {idx}")
            
            }
        };
        lines.push_str(line.as_str());
        lines.push('\n');
        pc += 1;
    }
    lines
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
        let bc = compile_expressions(vec![expression]);
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
        let bc = compile_expressions(vec![expression]);
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

        let bc = compile_expressions(program);

        assert_eq!(
            bc.constants,
            vec![
                ConstantValue::Object(ObjectValue::String("+".to_string())),
                ConstantValue::Integer(11),
                ConstantValue::Integer(12),
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
                ConstantValue::Object(ObjectValue::String("foo".to_string())),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::ReferenceGlobal.into(),
                0, // "+"
                Op::Constant.into(),
                1, // "11"
                Op::Constant.into(),
                2, // "12"
                Op::FuncCall.into(),
                2, // arity
                Op::DeclareGlobal.into(),
                3, // "foo" constant index
                Op::ReferenceGlobal.into(),
                4, // "foo" constant index
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
        let bc = compile_expressions(vec![SimpleExpression::RegularForm(vec![
            SimpleExpression::Symbol("*".to_string()),
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
        let bc = compile_expressions(vec![SimpleExpression::RegularForm(vec![
            SimpleExpression::Symbol("+".to_string()),
            SimpleExpression::RegularForm(vec![
                SimpleExpression::Symbol("+".to_string()),
                SimpleExpression::Constant(ConstantValue::Integer(11)),
                SimpleExpression::Constant(ConstantValue::Integer(12)),
            ]),
            SimpleExpression::RegularForm(vec![
                SimpleExpression::Symbol("*".to_string()),
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
                ConstantValue::Object(ObjectValue::String("*".to_string())),
                ConstantValue::Integer(13),
                ConstantValue::Integer(14),
            ]
        );

        assert_eq!(
            bc.code,
            vec![
                Op::ReferenceGlobal.into(),
                0, // reference function symbol (outer)
                Op::ReferenceGlobal.into(),
                1, // reference function symbol (inner)
                Op::Constant.into(),
                2, // load arg 1: "11"
                Op::Constant.into(),
                3, // load arg 2: "12"
                Op::FuncCall.into(),
                2, // call inner "+" with arity 2
                Op::ReferenceGlobal.into(),
                4, // load "*"
                Op::Constant.into(),
                5, // load arg 1: "13"
                Op::Constant.into(),
                6, // load arg 2: "14"
                Op::FuncCall.into(),
                2, // call function "*" (outer) with arity 2
                Op::FuncCall.into(),
                2, // call function "+" (outer) with arity 2
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
        let bc = compile_expressions(vec![SimpleExpression::Quote(Sexpr::List {
            quasiquote: false,
            sexprs: vec![Sexpr::Int(1), Sexpr::Int(2), Sexpr::Int(3)],
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
        let bc = compile_expressions(vec![SimpleExpression::Quote(Sexpr::List {
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
    }
}
        