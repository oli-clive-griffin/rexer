use crate::{
    compiler::{Expression, FunctionExpression},
    sexpr::SrcSexpr,
};

pub fn structure_sexpr(sexpr: &SrcSexpr, in_function: bool) -> Expression {
    match sexpr {
        // SrcSexpr::Symbol(_) => Expression::SrcSexpr(SrcSexpr::Symbol(a.clone())),
        // NOTE: might be better as Expression::Ref
        // that would seperate the concept of a reference from a symbol nicely
        SrcSexpr::List(sexprs) => {
            if let Some(special_form) = map_to_special_form(sexprs, in_function) {
                return special_form;
            }
            Expression::RegularForm(
                sexprs
                    .iter()
                    .map(|s| structure_sexpr(s, in_function))
                    .collect(),
            )
        }
        // self-eval
        v => Expression::SrcSexpr(v.clone()),
    }
}

fn map_to_special_form(sexprs: &[SrcSexpr], in_function: bool) -> Option<Expression> {
    let (head, rest) = sexprs.split_first().unwrap();

    if let SrcSexpr::Symbol(sym) = head {
        match sym.as_str() {
            "if" => {
                return Some(Expression::If {
                    condition: Box::new(structure_sexpr(&rest[0], in_function)),
                    then: Box::new(structure_sexpr(&rest[1], in_function)),
                    else_: Box::new(structure_sexpr(&rest[2], in_function)),
                });
            }
            "quote" => {
                if rest.len() != 1 {
                    panic!("quote expects 1 argument")
                }
                return Some(Expression::SrcSexpr(SrcSexpr::Quote(Box::new(
                    rest[0].clone(),
                ))));
            }
            "define" => {
                if rest.len() != 2 {
                    panic!("define expects 2 arguments, got {:?}", rest)
                }

                let name = match &rest[0] {
                    SrcSexpr::Symbol(s) => s.clone(),
                    _ => panic!("define expects symbol as first argument"),
                };

                let value = Box::new(structure_sexpr(&rest[1], in_function));
                return Some(if in_function {
                    Expression::LocalDefine { name, value }
                } else {
                    Expression::DeclareGlobal { name, value }
                });
            }
            "defun" => {
                let (signature, body_sexprs) = rest.split_first().unwrap();

                let (name, parameters) = match signature {
                    SrcSexpr::List(arg_sexprs) => {
                        let name = match &arg_sexprs[0] {
                            SrcSexpr::Symbol(s) => s.clone(),
                            _ => panic!("expected symbol for function name"),
                        };
                        let parameters = arg_sexprs[1..]
                            .iter()
                            .map(|sexpr| match sexpr {
                                SrcSexpr::Symbol(s) => s.clone(),
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

                let body_expressions = body_sexprs
                    .iter()
                    .map(|s| structure_sexpr(s, true))
                    .collect();

                return Some(if in_function {
                    Expression::LocalDefine {
                        name: name.clone(),
                        value: Box::new(Expression::FunctionLiteral(FunctionExpression::new(
                            parameters,
                            body_expressions,
                            Some(name),
                        ))),
                    }
                } else {
                    Expression::DeclareGlobal {
                        name: name.clone(),
                        value: Box::new(Expression::FunctionLiteral(FunctionExpression::new(
                            parameters,
                            body_expressions,
                            Some(name),
                        ))),
                    }
                    //     Expression::GlobalFunctionDeclaration {
                    //         name,
                    //         function_expr: FunctionExpression::new(parameters, body_expressions),
                    //     }
                    // });
                });
            }
            "fn" => {
                let (parameters, body_sexprs) = rest.split_first().unwrap();

                let parameters = match parameters {
                    SrcSexpr::List(arg_sexprs) => arg_sexprs
                        .iter()
                        .map(|sexpr| match sexpr {
                            SrcSexpr::Symbol(s) => s.clone(),
                            _ => panic!("expected symbol for parameter"),
                        })
                        .collect(),
                    _ => panic!("expected list for function parameters"),
                };

                let body_expressions = body_sexprs
                    .iter()
                    .map(|s| structure_sexpr(s, true))
                    .collect();

                return Some(Expression::FunctionLiteral(FunctionExpression::new(
                    parameters,
                    body_expressions,
                    None,
                )));
            }
            _ => {}
        }
    }
    None
}
