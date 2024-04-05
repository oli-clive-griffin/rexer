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
    fn eval(self, scope: &Scope) -> (Sexpr, Scope) {
        match self {
            Sexpr::List { sexprs, quasiquote } => eval_list(sexprs, scope, quasiquote),
            Sexpr::Symbol(sym) => (
                scope
                    .bindings
                    .get(&sym)
                    .unwrap_or_else(|| panic!("Symbol \"{}\" not found in scope: {:#?}", sym, scope))
                    .clone(),
                scope.clone(),
            ),
            _ => (self, scope.clone()),
        }
    }
}

fn eval_list(list: Vec<Sexpr>, scope: &Scope, quasiquote: bool) -> (Sexpr, Scope) {
    match (&list[0], quasiquote) {
        (_, true) => {
            let vals = list
                .iter()
                .map(|sexpr| match sexpr {
                    Sexpr::CommaUnquote(sexpr) => sexpr.clone().eval(scope).0, // TODO this feels wrong
                    sexpr => sexpr.clone(),
                })
                .collect::<Vec<Sexpr>>();

            (
                Sexpr::List {
                    sexprs: vals,
                    quasiquote: false,
                },
                scope.clone(),
            )
        }
        (Sexpr::Symbol(symbol), _) => match symbol.as_str() {
            "lambda" => eval_rest_as_lambda(&list[1..], scope),
            "macro" => eval_rest_as_macro_declaration(&list[1..], scope),
            "if" => eval_rest_as_if(&list[1..], scope),
            "let" => eval_rest_as_let(&list[1..], scope),
            "fn" => {
                let (result, scope) = eval_rest_as_function_declaration(&list[1..], scope);
                if let Sexpr::Function { parameters: _, body: _ } = &result {
                    let new_scope = scope.with_bindings(&[(symbol.clone(), result.clone())]);

                    (result, new_scope)
                } else {
                    panic!("fn must return a lambda")
                }
            }
            "quote" => {
                assert!(list.len() == 2, "quote must be called with one argument");
                (list[1].clone(), scope.clone())
            }
            _ => {
                let head = list[0].clone().eval(scope).0;

                eval_list(
                    iter::once(head)
                        .chain(list[1..].iter().cloned())
                        .collect::<Vec<Sexpr>>(),
                    scope,
                    false,
                )
            }
        },
        (Sexpr::Function { parameters, body }, _) => {
            let arguments = list[1..]
                .iter()
                .cloned()
                .map(|arg| arg.eval(scope).0)
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
            sequential_eval(body.clone().to_vec(), &func_scope)
            // body.clone().eval(&func_scope)
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
            let expanded = body.clone().eval(macro_scope).0; // evaluate the macro
            (expanded.eval(scope).0, scope.clone()) // evaluate the result of the macro in the original scope
        }
        (Sexpr::List { sexprs, quasiquote }, _) => eval_list(
            iter::once(sexprs[0].clone().eval(scope).0)
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
                .map(|arg| arg.eval(scope).0)
                .collect::<Vec<Sexpr>>();
            (builtin.eval(&arguments), scope.clone())
        }
        (Sexpr::CommaUnquote(_), _) => panic!("Unquote outside of quasiquoted context"),
    }
}

fn eval_rest_as_function_declaration(rest: &[Sexpr], scope: &Scope) -> (Sexpr, Scope) {
    match &rest[0] {
        Sexpr::List { quasiquote, sexprs } => {
            if *quasiquote {
                panic!("quasiquote not allowed in function arguments");
            }

            // (<function_name> <arg1> <arg2>)
            if let Sexpr::Symbol(func_name) = &sexprs[0] {

                let arg_names = sexprs[1..].iter().map(|expr| {
                    match expr {
                        Sexpr::Symbol(name) => name.clone(),
                        _ => panic!("Function arguments must be identifiers"),
                    }
                }).collect::<Vec<String>>();

                let function = Sexpr::Function {
                    parameters: arg_names,
                    body: Box::new(rest[1..].to_vec()),
                };

                let new_scope = scope.with_bindings(&[(func_name.clone(), function.clone())]);
                return (function, new_scope.clone());
            } else {
                panic!("Function declaration must start with a symbol");
                
            }
        }
        _ => panic!("Function declaration must have a list of arguments"),
    }
}

fn eval_rest_as_lambda(rest: &[Sexpr], scope: &Scope) -> (Sexpr, Scope) {
    let args = parse_as_args(&rest[0]);
    let fn_body = rest[1..].to_vec();

    // TODO closures ???
    // substitute scope into fn_body ???
    // actually should be easy as everything is pure and passed by value
    let _ = scope;

    (
        Sexpr::Function {
            parameters: args,
            body: Box::new(fn_body),
        },
        scope.clone(),
    )
}

fn eval_rest_as_macro_declaration(rest: &[Sexpr], scope: &Scope) -> (Sexpr, Scope) {
    let args = parse_as_args(&rest[0]);
    let fn_body = &rest[1];

    // todo substitute scope into fn_body
    let _ = scope;

    (
        Sexpr::Macro {
            parameters: args,
            body: Box::new(fn_body.clone()),
        },
        scope.clone(),
    )
}

fn eval_rest_as_let(rest: &[Sexpr], scope: &Scope) -> (Sexpr, Scope) {
    let binding_exprs = rest[..rest.len() - 1].to_vec();
    let expr = rest.last().expect("let must have a body");
    let bindings = generate_let_bindings(binding_exprs, scope);
    expr.clone().eval(&scope.with_bindings(&bindings))
}

fn eval_rest_as_if(rest: &[Sexpr], scope: &Scope) -> (Sexpr, Scope) {
    if rest.len() != 3 {
        panic!("malformed if statement: Must have 3 arguments");
    }
    let condition = rest[0].clone();
    let if_body = rest[1].clone();
    let else_body = rest[2].clone();

    // TODO: encapse this is Sexpr.bool()
    if let Sexpr::Bool(cond) = condition.eval(scope).0 {
        (if cond { if_body } else { else_body }).eval(scope)
    } else {
        panic!("If condition must be a boolean");
    }
}

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
                    let val = sexprs[1].clone().eval(scope).0;
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

fn sequential_eval(list: Vec<Sexpr>, scope: &Scope) -> (Sexpr, Scope) {
    let mut scope = scope.clone();
    let mut i = 0;
    loop {
        let (res, new_scope) = list[i].clone().eval(&scope);
        scope = new_scope;
        i += 1;
        if i == list.len() {
            return (res, scope);
        }
    }

}

/// for now, assume that the AST is a single SExpr
/// and just evaluate it.
/// Obvious next steps are to allow for multiple SExprs (lines)
/// and to manage a global scope being passed between them.
pub fn evaluate(ast: Ast) {
    sequential_eval(ast.expressions, &Scope::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let expr = Sexpr::List {
            sexprs: vec![Sexpr::Symbol("+".to_string()), Sexpr::Int(1), Sexpr::Int(2)],
            quasiquote: false,
        };
        let output = expr.eval(&Scope::new()).0;
        assert_eq!(output, Sexpr::Int(3));
    }

    #[test]
    fn test2() {
        let sexpr = Sexpr::List {
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
        };
        let res = sexpr.eval(&Scope::new()).0;
        assert_eq!(res, Sexpr::Int(11))
    }

    #[test]
    fn test3() {
        let sexpr = Sexpr::List {
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
        };
        let res = sexpr.eval(&Scope::new()).0;
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
        let res = ast.eval(&Scope::new()).0;
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
        let res = ast.eval(&Scope::new()).0;
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
        assert_eq!(ast.eval(&Scope::new()).0, Sexpr::Int(3))
    }
}
