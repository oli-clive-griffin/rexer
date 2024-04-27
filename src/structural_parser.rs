use crate::{
    compiler::{Expression, GlobalFunctionDeclaration},
    sexpr::Sexpr,
    vm::{ConstantValue, ObjectValue},
};

pub fn structure_sexpr(sexpr: &Sexpr) -> Expression {
    match sexpr {
        Sexpr::Symbol(sym) => match sym.as_str() {
            "nil" => Expression::Constant(ConstantValue::Nil),
            _ => Expression::Symbol(sym.clone()),
        },
        Sexpr::String(str) => {
            Expression::Constant(ConstantValue::Object(ObjectValue::String(str.clone())))
        }
        Sexpr::Bool(bool) => Expression::Constant(ConstantValue::Boolean(*bool)),
        Sexpr::Int(i) => Expression::Constant(ConstantValue::Integer(*i)),
        Sexpr::Float(f) => Expression::Constant(ConstantValue::Float(*f)),
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
        Sexpr::Nil => Expression::Constant(ConstantValue::Nil),
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
            Expression::RegularForm(sexprs.iter().map(structure_sexpr).collect())
        }
    }
}

fn map_to_special_form(sexprs: &[Sexpr]) -> Option<Expression> {
    let (head, rest) = sexprs.split_first().unwrap();

    if let Sexpr::Symbol(sym) = head {
        match sym.as_str() {
            "if" => {
                return Some(Expression::If {
                    condition: Box::new(structure_sexpr(&rest[0])),
                    then: Box::new(structure_sexpr(&rest[1])),
                    else_: Box::new(structure_sexpr(&rest[2])),
                });
            }
            "set!" => {
                let name = match &rest[0] {
                    Sexpr::Symbol(s) => s,
                    _ => panic!("set! expects symbol as first argument"),
                };
                return Some(Expression::DeclareGlobal {
                    name: name.to_string(),
                    value: Box::new(structure_sexpr(&rest[1])),
                });
            }
            "quote" => {
                if rest.len() != 1 {
                    panic!("quote expects 1 argument")
                }
                return Some(Expression::Quote(rest[1].clone()));
            }
            "define" => {
                if rest.len() != 2 {
                    panic!("define expects 2 arguments")
                }
                return Some(Expression::Define {
                    name: match &rest[0] {
                        Sexpr::Symbol(s) => s.clone(),
                        _ => panic!("define expects symbol as first argument"),
                    },
                    value: Box::new(structure_sexpr(&rest[1])),
                });
            }
            "fn" => {
                let (signature, body_sexprs) = rest.split_first().unwrap();

                let (name, parameters) = match signature {
                    Sexpr::List {
                        quasiquote,
                        sexprs: arg_sexprs,
                    } => {
                        if *quasiquote {
                            todo!("inappropriate quasiquote")
                        }
                        let name = match &arg_sexprs[0] {
                            Sexpr::Symbol(s) => s.clone(),
                            _ => panic!("expected symbol for function name"),
                        };
                        let parameters = arg_sexprs[1..]
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

                let body_expressions = body_sexprs.iter().map(structure_sexpr).collect();

                return Some(Expression::GlobalFunctionDeclaration(Box::new(
                    GlobalFunctionDeclaration::new(name, parameters, body_expressions),
                )));
            }
            _ => {}
        }
    }
    None
}
