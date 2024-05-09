use crate::{
    compiler::{Expression, FunctionExpression},
    parser::Ast,
    sexpr::SrcSexpr,
};

pub fn structure_ast(ast: Ast) -> Vec<Expression> {
    ast.expressions
        .iter()
        .map(structure_root_sexpr)
        .collect::<Vec<Expression>>()
}

fn structure_root_sexpr(sexpr: &SrcSexpr) -> Expression {
    structure_sexpr(sexpr, false, true)
}

fn structure_sexpr(sexpr: &SrcSexpr, in_function: bool, discarding: bool) -> Expression {
    match sexpr {
        // SrcSexpr::Symbol(_) => Expression::SrcSexpr(SrcSexpr::Symbol(a.clone())),
        // NOTE: might be better as Expression::Ref
        // that would seperate the concept of a reference from a symbol nicely
        SrcSexpr::List(sexprs) => {
            if let Some(special_form) = map_to_special_form(sexprs, in_function, discarding) {
                return special_form;
            }
            let regular_form = Expression::RegularForm(
                sexprs
                    .iter()
                    .map(|s| structure_sexpr(s, in_function, false)) // don't discard the last one
                    .collect(),
            );
            if discarding {
                return Expression::Discard(Box::new(regular_form));
            }
            return regular_form;
        }
        // self-eval
        v => {
            let x = Expression::SrcSexpr(v.clone());
            if discarding {
                return Expression::Discard(Box::new(x));
            }
            return x;
        }
    }
}

fn map_to_special_form(
    sexprs: &[SrcSexpr],
    in_function: bool,
    // if true, the result will be wrapped in a Discard if applicable
    // for example, a `define` will not be implcated, but an `if` will
    discarding: bool,
) -> Option<Expression> {
    let (head, rest) = sexprs.split_first().unwrap();

    if let SrcSexpr::Symbol(sym) = head {
        match sym.as_str() {
            "if" => {
                let expr = Expression::If {
                    condition: Box::new(structure_sexpr(&rest[0], in_function, false)),
                    then: Box::new(structure_sexpr(&rest[1], in_function, false)),
                    else_: Box::new(structure_sexpr(&rest[2], in_function, false)),
                };

                return Some(optionally_wrap_discard(expr, discarding));
            }
            "quote" => {
                if rest.len() != 1 {
                    panic!("quote expects 1 argument")
                }
                let expr = Expression::SrcSexpr(SrcSexpr::Quote(Box::new(rest[0].clone())));
                return Some(optionally_wrap_discard(expr, discarding));
            }
            "define" => {
                if rest.len() != 2 {
                    panic!("define expects 2 arguments, got {:?}", rest)
                }

                let name = match &rest[0] {
                    SrcSexpr::Symbol(s) => s.clone(),
                    _ => panic!("define expects symbol as first argument"),
                };

                let value = Box::new(structure_sexpr(&rest[1], in_function, false));

                // ignore discarding as define doesn't evaluate to a stackval
                return Some(if in_function {
                    Expression::LocalDefine { name, value }
                } else {
                    Expression::DeclareGlobal { name, value }
                });
            }
            "set" => {
                if rest.len() != 2 {
                    panic!("define expects 2 arguments, got {:?}", rest)
                }

                let name = match &rest[0] {
                    SrcSexpr::Symbol(s) => s.clone(),
                    _ => panic!("define expects symbol as first argument"),
                };

                let value = Box::new(structure_sexpr(&rest[1], in_function, false));

                if !in_function {
                    panic!("set! is only allowed in function bodies")
                }

                return Some(Expression::LocalSet { name, value });
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

                let body_expressions = compile_sequential_expressions(body_sexprs);

                let name = name.clone();
                let value = Box::new(Expression::FunctionLiteral(FunctionExpression::new(
                    parameters,
                    body_expressions,
                    Some(name.clone()),
                )));

                // ignore discarding as define doesn't evaluate to a stackval
                return Some(if in_function {
                    Expression::LocalDefine { name, value }
                } else {
                    Expression::DeclareGlobal { name, value }
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

                let body_expressions = compile_sequential_expressions(body_sexprs);

                let function_literal = Expression::FunctionLiteral(FunctionExpression::new(
                    parameters,
                    body_expressions,
                    None,
                ));
                return Some(optionally_wrap_discard(function_literal, discarding));
            }
            _ => {}
        }
    }
    None
}

fn compile_sequential_expressions(sexprs: &[SrcSexpr]) -> Vec<Expression> {
    sexprs
        .iter()
        .enumerate()
        .map(|(i, s)| structure_sexpr(s, true, i != sexprs.len() - 1)) // todo really?
        .collect()
}

fn optionally_wrap_discard(expr: Expression, discarding: bool) -> Expression {
    if discarding {
        return Expression::Discard(Box::new(expr));
    }
    expr
}

#[cfg(test)]
mod tests {
    use crate::sexpr::SrcSexpr;

    use super::*;

    #[test]
    fn test1() {
        let sexpr = SrcSexpr::List(vec![
            SrcSexpr::Symbol("if".to_string()),
            SrcSexpr::List(vec![
                SrcSexpr::Symbol("<".to_string()),
                SrcSexpr::Int(1),
                SrcSexpr::Int(2),
            ]),
            SrcSexpr::Int(1),
            SrcSexpr::Int(2),
        ]);

        let expected = Expression::If {
            condition: Box::new(Expression::RegularForm(vec![
                Expression::SrcSexpr(SrcSexpr::Symbol("<".to_string())),
                Expression::SrcSexpr(SrcSexpr::Int(1)),
                Expression::SrcSexpr(SrcSexpr::Int(2)),
            ])),
            then: Box::new(Expression::SrcSexpr(SrcSexpr::Int(1))),
            else_: Box::new(Expression::SrcSexpr(SrcSexpr::Int(2))),
        };

        assert_eq!(structure_sexpr(&sexpr, false, false), expected);
    }

    #[test]
    fn test2() {
        let ast = Ast {
            expressions: vec![SrcSexpr::String("discard me".to_string())],
        };

        let expected = vec![Expression::Discard(Box::new(Expression::SrcSexpr(
            SrcSexpr::String("discard me".to_string()),
        )))];

        assert_eq!(structure_ast(ast), expected);
    }

    #[test]
    fn test3() {
        let ast = Ast {
            expressions: vec![SrcSexpr::List(vec![
                SrcSexpr::Symbol("define".to_string()),
                SrcSexpr::Symbol("x".to_string()),
                SrcSexpr::Int(1),
            ])],
        };

        let expected = vec![Expression::DeclareGlobal {
            name: "x".to_string(),
            value: Box::new(Expression::SrcSexpr(SrcSexpr::Int(1))),
        }];

        assert_eq!(structure_ast(ast), expected);
    }

    #[test]
    fn test4() {
        let ast = Ast {
            expressions: vec![SrcSexpr::List(vec![
                SrcSexpr::Symbol("defun".to_string()),
                SrcSexpr::List(vec![
                    SrcSexpr::Symbol("f".to_string()),
                    SrcSexpr::Symbol("x".to_string()),
                ]),
                SrcSexpr::String("discard me".to_string()),
                SrcSexpr::String("return me".to_string()),
            ])],
        };

        let expected = vec![Expression::DeclareGlobal {
            name: "f".to_string(),
            value: Box::new(Expression::FunctionLiteral(FunctionExpression {
                name: Some("f".to_string()),
                parameters: vec!["x".to_string()],
                body: vec![
                    Expression::Discard(Box::new(Expression::SrcSexpr(SrcSexpr::String(
                        "discard me".to_string(),
                    )))),
                    Expression::SrcSexpr(SrcSexpr::String("return me".to_string())),
                ],
            })),
        }];

        assert_eq!(structure_ast(ast), expected);
    }

    #[test]
    fn test5() {
        let ast = Ast {
            expressions: vec![SrcSexpr::List(vec![
                SrcSexpr::Symbol("print".to_string()),
                SrcSexpr::String("hello, world".to_string()),
            ])],
        };

        let expected = vec![Expression::Discard(Box::new(Expression::RegularForm(
            vec![
                Expression::SrcSexpr(SrcSexpr::Symbol("print".to_string())),
                Expression::SrcSexpr(SrcSexpr::String("hello, world".to_string())),
            ],
        )))];

        assert_eq!(structure_ast(ast), expected);
    }
}
