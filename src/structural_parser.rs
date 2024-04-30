use crate::{
    compiler::{Expression, FunctionExpression},
    sexpr::SrcSexpr,
};

pub fn structure_sexpr(sexpr: &SrcSexpr) -> Expression {
    match sexpr {
        SrcSexpr::Symbol(a) => Expression::SrcSexpr(SrcSexpr::Symbol(a.clone())),
        SrcSexpr::String(a) => Expression::SrcSexpr(SrcSexpr::String(a.clone())),
        SrcSexpr::Bool(a) => Expression::SrcSexpr(SrcSexpr::Bool(*a)),
        SrcSexpr::Integer(a) => Expression::SrcSexpr(SrcSexpr::Integer(*a)),
        SrcSexpr::Float(a) => Expression::SrcSexpr(SrcSexpr::Float(*a)),
        SrcSexpr::Quote(sexpr) => Expression::SrcSexpr(SrcSexpr::Quote(Box::new(*sexpr.clone()))),
        SrcSexpr::List(sexprs) => {
            if let Some(special_form) = map_to_special_form(sexprs) {
                return special_form;
            }
            Expression::RegularForm(sexprs.iter().map(structure_sexpr).collect())
        }

    }
}

fn map_to_special_form(sexprs: &[SrcSexpr]) -> Option<Expression> {
    let (head, rest) = sexprs.split_first().unwrap();

    if let SrcSexpr::Symbol(sym) = head {
        match sym.as_str() {
            "if" => {
                return Some(Expression::If {
                    condition: Box::new(structure_sexpr(&rest[0])),
                    then: Box::new(structure_sexpr(&rest[1])),
                    else_: Box::new(structure_sexpr(&rest[2])),
                });
            }
            "quote" => {
                if rest.len() != 1 {
                    panic!("quote expects 1 argument")
                }
                return Some(Expression::SrcSexpr(SrcSexpr::Quote(Box::new(
                    rest[1].clone(),
                ))));
            }
            "define" => {
                if rest.len() != 2 {
                    panic!("define expects 2 arguments, got {:?}", rest)
                }
                return Some(Expression::Define {
                    name: match &rest[0] {
                        SrcSexpr::Symbol(s) => s.clone(),
                        _ => panic!("define expects symbol as first argument"),
                    },
                    value: Box::new(structure_sexpr(&rest[1])),
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

                let body_expressions = body_sexprs.iter().map(structure_sexpr).collect();

                return Some(Expression::GlobalFunctionDeclaration {
                    name,
                    function_expr: FunctionExpression::new(parameters, body_expressions),
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

                let body_expressions = body_sexprs.iter().map(structure_sexpr).collect();

                return Some(Expression::FunctionLiteral(FunctionExpression::new(
                    parameters,
                    body_expressions,
                )));
            }
            _ => {}
        }
    }
    None
}
