use core::panic;
use std::collections::HashMap;
use std::iter;

use crate::builtins::BUILTINTS;
use crate::parser::{Ast, Sexpr};

#[derive(Debug, Clone, PartialEq)]
struct Scope {
    bindings: HashMap<String, Sexpr>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            bindings: HashMap::from_iter(
                BUILTINTS.map(|builtin| (builtin.symbol.to_string(), Sexpr::BuiltIn(builtin))),
            ),
        }
    }

    fn with_bindings(&self, bindings: &[(String, Sexpr)]) -> Scope {
        let mut new_bindings = self.bindings.clone();
        new_bindings.extend(bindings.iter().cloned());

        Scope {
            bindings: new_bindings,
        }
    }
}

impl Sexpr {
    fn eval(self, scope: &Scope) -> Sexpr {
        match self {
            Sexpr::List { sexprs, quasiquote } => eval_list(sexprs, scope, quasiquote),
            Sexpr::Symbol(sym) => scope
                .bindings
                .get(&sym)
                .unwrap_or_else(|| panic!("Symbol not found in scope: {}", sym))
                .clone(),
            _ => self,
        }
    }
}

fn eval_list(list: Vec<Sexpr>, scope: &Scope, quasiquote: bool) -> Sexpr {
    match (&list[0], quasiquote) {
        (_, true) => {
            let vals = list
                .iter()
                .map(|sexpr| match sexpr {
                    Sexpr::CommaUnquote(sexpr) => sexpr.clone().eval(scope),
                    sexpr => sexpr.clone(),
                })
                .collect::<Vec<Sexpr>>();

            Sexpr::List {
                sexprs: vals,
                quasiquote: false,
            }
        }
        (Sexpr::Symbol(symbol), _) => match symbol.as_str() {
            "lambda" => eval_rest_as_function_declaration(&list[1..], scope),
            "macro" => eval_rest_as_macro_declaration(&list[1..], scope),
            "if" => eval_rest_as_if(&list[1..], scope),
            "let" => eval_rest_as_let(&list[1..], scope),
            "quote" => {
                assert!(list.len() == 2, "quote must be called with one argument");
                list[1].clone()
            }
            _ => {
                let head = list[0].clone().eval(scope);

                eval_list(
                    iter::once(head)
                        .chain(list[1..].iter().cloned())
                        .collect::<Vec<Sexpr>>(),
                    scope,
                    false,
                )
            }
        },
        (Sexpr::Lambda { parameters, body }, _) => {
            let arguments = list[1..]
                .iter()
                .cloned()
                .map(|arg| arg.eval(scope))
                .collect::<Vec<Sexpr>>();

            if parameters.len() != arguments.len() {
                panic!("Function called with incorrect number of arguments");
            }

            // zip the args and params together
            let bindings = (*parameters)
                .iter()
                .cloned()
                .zip(arguments.iter().cloned())
                .collect::<Vec<(String, Sexpr)>>();

            let func_scope = scope.with_bindings(&bindings);
            body.clone().eval(&func_scope)
        }
        (Sexpr::Macro { parameters, body }, _) => {
            // DON'T EVALUATE THE MACRO BODY
            let arguments = &list[1..];

            if parameters.len() != arguments.len() {
                panic!("Macro called with incorrect number of arguments");
            }

            // zip the args and params together
            // "parameters" is now a list of strings which refer to the **un-evaluated** arguments
            // i.e. (macro (switch a b) (list b a)
            //      (switch 1 x) -> { a: Int(1), b: Symbol("x") }
            let macro_bindings = parameters
                .iter()
                .cloned()
                .zip(arguments.iter().cloned())
                .collect::<Vec<(String, Sexpr)>>();

            // create a new scope with the macro_bindings for inside the macro
            let macro_scope = &scope.with_bindings(&macro_bindings);
            let expanded = body.clone().eval(macro_scope); // evaluate the macro
            expanded.eval(scope) // evaluate the result of the macro in the original scope
        }
        (Sexpr::List { sexprs, quasiquote }, _) => eval_list(
            iter::once(sexprs[0].clone().eval(scope))
                .chain(sexprs[1..].iter().cloned())
                .collect::<Vec<Sexpr>>(),
            scope,
            *quasiquote, // TODO revise
        ),
        (Sexpr::String(_), _) => panic!("Cannot call string value"),
        (Sexpr::Bool(_), _) => panic!("Cannot call boolean value"),
        (Sexpr::Int(_), _) => panic!("Cannot call int value"),
        (Sexpr::Float(_), _) => panic!("Cannot call float value"),
        (Sexpr::BuiltIn(builtin), _) => {
            let arguments = list[1..]
                .iter()
                .cloned()
                .map(|arg| arg.eval(scope))
                .collect::<Vec<Sexpr>>();
            builtin.eval(&arguments)
        }
        (Sexpr::CommaUnquote(_), _) => panic!("Unquote outside of quasiquoted context"),
    }
}

fn eval_rest_as_function_declaration(rest: &[Sexpr], scope: &Scope) -> Sexpr {
    let args = parse_as_args(&rest[0]);
    let fn_body = &rest[1];

    // TODO closures ???
    // substitute scope into fn_body ???
    // actually should be easy as everything is pure and passed by value
    let _ = scope;

    Sexpr::Lambda {
        parameters: args,
        body: Box::new(fn_body.clone()),
    }
}

fn eval_rest_as_macro_declaration(rest: &[Sexpr], scope: &Scope) -> Sexpr {
    let args = parse_as_args(&rest[0]);
    let fn_body = &rest[1];

    // todo substitute scope into fn_body
    let _ = scope;

    Sexpr::Macro {
        parameters: args,
        body: Box::new(fn_body.clone()),
    }
}

fn eval_rest_as_let(rest: &[Sexpr], scope: &Scope) -> Sexpr {
    let binding_exprs = rest[..rest.len() - 1].to_vec();
    let expr = rest.last().expect("let must have a body");
    let bindings = generate_let_bindings(binding_exprs, scope);
    expr.clone().eval(&scope.with_bindings(&bindings))
}

fn eval_rest_as_if(rest: &[Sexpr], scope: &Scope) -> Sexpr {
    if rest.len() != 3 {
        panic!("malformed if statement: Must have 3 arguments");
    }
    let condition = rest[0].clone();
    let if_body = rest[1].clone();
    let else_body = rest[2].clone();

    // TODO: encapse this is Sexpr.bool()
    if let Sexpr::Bool(cond) = condition.eval(scope) {
        (if cond { if_body } else { else_body }).eval(scope)
    } else {
        panic!("If condition must be a boolean");
    }
}

fn eval_rest_as_quote(list: &[Sexpr]) -> Sexpr {
    if list.len() != 1 {
        panic!("quote must be called with one argument");
    }

    // quote is a special form that just returns the argument
    list[0].clone()
}

// fn quote(node: Sexpr) -> Sexpr {
//     match node {
//         Sexpr::List { sexprs, quasiquote } => Sexpr::List {
//             sexprs: sexprs.iter().map(|node| quote(node.clone())).collect(),
//             quasiquote,
//         },
//         Sexpr::Lambda {
//             parameters: _,
//             body: _,
//         } => {
//             panic!("this shouldn't happen (quoting a Lambda value) as the user has no way to input a raw value of this kind")
//         }
//         Sexpr::Macro {
//             parameters: _,
//             body: _,
//         } => {
//             panic!("this shouldn't happen (quoting a Macro value) as the user has no way to input a raw value of this kind")
//         }
//         Sexpr::BuiltIn(_) => {
//             panic!("this shouldn't happen (quoting a BuiltIn value) as the user has no way to input a raw value of this kind")
//         }
//         Sexpr::Symbol(_) | Sexpr::String(_) | Sexpr::Bool(_) | Sexpr::Int(_) | Sexpr::Float(_) => {
//             node
//         }
//         Sexpr::CommaUnquote(_) => {

//     }
// }

fn generate_let_bindings(list: Vec<Sexpr>, scope: &Scope) -> Vec<(String, Sexpr)> {
    list.iter()
        .cloned()
        .map(|node| match node {
            Sexpr::List { quasiquote, sexprs } => {
                if quasiquote {
                    panic!("quasiquote not allowed in let bindings");
                }
                if sexprs.len() != 2 {
                    panic!("let binding must be a list of two elements");
                }
                if let Sexpr::Symbol(ident) = &sexprs[0] {
                    let val = sexprs[1].clone().eval(scope);
                    (ident.clone(), val.clone())
                } else {
                    panic!("left side of let binding must be an identifier");
                }
            }
            _ => panic!("All bindings must be lists"),
        })
        .collect::<Vec<(String, Sexpr)>>()
}

fn parse_as_args(expr: &Sexpr) -> Vec<String> {
    if let Sexpr::List { sexprs, quasiquote } = expr {
        if *quasiquote {
            panic!("quasiquote not allowed in function arguments");
        }
        sexprs
            .iter()
            .map(|e| {
                if let Sexpr::Symbol(ident) = e {
                    ident.clone()
                } else {
                    panic!("Function arguments must be identifiers")
                }
            })
            .collect::<Vec<String>>()
    } else {
        panic!("Function arguments must be a list")
    }
}

/// for now, assume that the AST is a single SExpr
/// and just evaluate it.
/// Obvious next steps are to allow for multiple SExprs (lines)
/// and to manage a global scope being passed between them.
pub fn evaluate(ast: Ast) -> Sexpr {
    println!("{:#?}", ast);
    ast.root.eval(&Scope::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let ast = Ast {
            root: Sexpr::List {
                sexprs: vec![Sexpr::Symbol("+".to_string()), Sexpr::Int(1), Sexpr::Int(2)],
                quasiquote: false,
            },
        };
        let output = ast.root.eval(&Scope::new());
        assert_eq!(output, Sexpr::Int(3));
    }

    #[test]
    fn test2() {
        let ast = Ast {
            root: Sexpr::List {
                quasiquote: false,
                sexprs: vec![
                    Sexpr::Symbol("+".to_string()),
                    Sexpr::Int(1),
                    Sexpr::Int(2),
                    Sexpr::List {
                        quasiquote: false,
                        sexprs: vec![Sexpr::Symbol("-".to_string()), Sexpr::Int(4), Sexpr::Int(3)],
                    },
                    Sexpr::Int(5),
                    Sexpr::List {
                        quasiquote: false,
                        sexprs: vec![
                            Sexpr::Symbol("*".to_string()),
                            Sexpr::Int(1),
                            // Sexpr::Float(2.3),
                            Sexpr::Int(2),
                        ],
                    },
                ],
            },
        };
        let res = ast.root.eval(&Scope::new());
        assert_eq!(res, Sexpr::Int(11))
    }

    #[test]
    fn test3() {
        let ast = Ast {
            root: Sexpr::List {
                quasiquote: false,
                sexprs: vec![
                    Sexpr::Symbol("let".to_string()),
                    Sexpr::List {
                        quasiquote: false,
                        sexprs: vec![Sexpr::Symbol("x".to_string()), Sexpr::Int(2)],
                    },
                    Sexpr::List {
                        quasiquote: false,
                        sexprs: vec![
                            Sexpr::Symbol("*".to_string()),
                            Sexpr::Symbol("x".to_string()),
                            Sexpr::Int(3),
                        ],
                    },
                ],
            },
        };
        let res = ast.root.eval(&Scope::new());
        assert_eq!(res, Sexpr::Int(6))
    }

    #[test]
    fn test_macros_1() {
        /*
         * (let
         *   (switch (macro (a b) (quote (b a))))
         *   (switch 3 inc))
         */
        let ast = Sexpr::List {
            quasiquote: false,
            sexprs: vec![
                Sexpr::Symbol("let".to_string()),
                Sexpr::List {
                    quasiquote: false,
                    sexprs: vec![
                        Sexpr::Symbol("switch".to_string()),
                        Sexpr::List {
                            quasiquote: false,
                            sexprs: vec![
                                Sexpr::Symbol("macro".to_string()),
                                Sexpr::List {
                                    quasiquote: false,
                                    sexprs: vec![
                                        Sexpr::Symbol("a".to_string()),
                                        Sexpr::Symbol("b".to_string()),
                                    ],
                                },
                                Sexpr::List {
                                    quasiquote: false,
                                    sexprs: vec![
                                        Sexpr::Symbol("list".to_string()),
                                        Sexpr::Symbol("b".to_string()),
                                        Sexpr::Symbol("a".to_string()),
                                    ],
                                },
                            ],
                        },
                    ],
                },
                Sexpr::List {
                    quasiquote: false,
                    sexprs: vec![
                        Sexpr::Symbol("switch".to_string()),
                        Sexpr::Int(1),
                        Sexpr::Symbol("inc".to_string()),
                    ],
                },
            ],
        };
        let res = ast.eval(&Scope::new());
        assert_eq!(res, Sexpr::Int(2))
    }

    #[test]
    fn test_macros_2() {
        /*
         * (let
         *   (switch (macro (a b) (quote (b a))))
         *   (switch 3 inc))
         */
        let ast = Sexpr::List {
            quasiquote: false,
            sexprs: vec![
                Sexpr::Symbol("let".to_string()),
                Sexpr::List {
                    quasiquote: false,
                    sexprs: vec![
                        Sexpr::Symbol("switch".to_string()),
                        Sexpr::List {
                            quasiquote: false,
                            sexprs: vec![
                                Sexpr::Symbol("macro".to_string()),
                                Sexpr::List {
                                    quasiquote: false,
                                    sexprs: vec![
                                        Sexpr::Symbol("a".to_string()),
                                        Sexpr::Symbol("b".to_string()),
                                    ],
                                },
                                Sexpr::List {
                                    quasiquote: false,
                                    sexprs: vec![
                                        Sexpr::Symbol("list".to_string()),
                                        Sexpr::Symbol("b".to_string()),
                                        Sexpr::Symbol("a".to_string()),
                                    ],
                                },
                            ],
                        },
                    ],
                },
                Sexpr::List {
                    quasiquote: false,
                    sexprs: vec![
                        Sexpr::Symbol("switch".to_string()),
                        Sexpr::Int(1),
                        Sexpr::Symbol("inc".to_string()),
                    ],
                },
            ],
        };
        let res = ast.eval(&Scope::new());
        assert_eq!(res, Sexpr::Int(2))
    }

    #[test]
    fn test_macros_3() {
        /*
         * (let
         *   (infix (macro (a op b) (list op a b)))
         *   (infix 1 + 2))
         */
        let ast = Sexpr::List {
            quasiquote: false,
            sexprs: vec![
                Sexpr::Symbol("let".to_string()),
                Sexpr::List {
                    quasiquote: false,
                    sexprs: vec![
                        Sexpr::Symbol("infix".to_string()),
                        Sexpr::List {
                            quasiquote: false,
                            sexprs: vec![
                                Sexpr::Symbol("macro".to_string()),
                                Sexpr::List {
                                    quasiquote: false,
                                    sexprs: vec![
                                        Sexpr::Symbol("a".to_string()),
                                        Sexpr::Symbol("op".to_string()),
                                        Sexpr::Symbol("b".to_string()),
                                    ],
                                },
                                Sexpr::List {
                                    quasiquote: false,
                                    sexprs: vec![
                                        Sexpr::Symbol("list".to_string()),
                                        Sexpr::Symbol("op".to_string()),
                                        Sexpr::Symbol("a".to_string()),
                                        Sexpr::Symbol("b".to_string()),
                                    ],
                                },
                            ],
                        },
                    ],
                },
                Sexpr::List {
                    quasiquote: false,
                    sexprs: vec![
                        Sexpr::Symbol("infix".to_string()),
                        Sexpr::Int(1),
                        Sexpr::Symbol("+".to_string()),
                        Sexpr::Int(2),
                    ],
                },
            ],
        };
        assert_eq!(ast.eval(&Scope::new()), Sexpr::Int(3))
    }
}
