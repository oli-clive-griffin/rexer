use std::collections::HashMap;
use std::fmt::Display;
use std::iter;

use crate::builtins::BUILT_INS;
use crate::parser::Ast;
use crate::sexpr::Sexpr;

#[derive(Debug, Clone, PartialEq)]
struct Scope {
    bindings: HashMap<String, Sexpr>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            bindings: HashMap::from_iter(BUILT_INS.map(|builtin| {
                (
                    builtin.symbol.to_string(),
                    Sexpr::BuiltIn(builtin),
                )
            })),
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
    fn eval(self, scope: &Scope) -> Result<(Sexpr, Scope), String> {
        match self {
            Sexpr::List(sexprs) => eval_list(sexprs, scope),
            Sexpr::QuasiQuotedList(sexprs) => {
                let res = sexprs
                    .into_iter()
                    .map(|sexpr| match sexpr {
                        Sexpr::CommaUnquote(sexpr) => {
                            sexpr.clone().eval(scope).map(|r| r.0)
                        }
                        sexpr => Ok(sexpr),
                    })
                    .collect::<Result<Vec<Sexpr>, String>>()?;

                Ok((Sexpr::List(res), scope.clone()))
            }
            Sexpr::Quote(sexpr) => Ok((*sexpr, scope.clone())),
            Sexpr::Symbol(sym) => match sym.as_str() {
                "nil" => Ok((Sexpr::Nil, scope.clone())),
                _ => match scope.bindings.get(&sym) {
                    Some(sexpr) => Ok((sexpr.clone(), scope.clone())),
                    None => Err(format!("Symbol {} not found in scope", sym)),
                },
            },
            // self-evaluating S-expressions
            Sexpr::String(_)
            | Sexpr::Bool(_)
            | Sexpr::Int(_)
            | Sexpr::Float(_)
            | Sexpr::BuiltIn(_)
            | Sexpr::CommaUnquote(_)
            | Sexpr::Macro {
                parameters: _,
                body: _,
            }
            | Sexpr::Function {
                parameters: _,
                body: _,
            }
            | Sexpr::Nil => Ok((self, scope.clone())),
        }
    }
}

fn eval_list(
    list: Vec<Sexpr>,
    scope: &Scope,
) -> Result<(Sexpr, Scope), String> {
    if list.is_empty() {
        return Ok((Sexpr::List(vec![]), scope.clone()));
    }

    let (first, rest) = list.split_first().unwrap();

    // FIXME fix this clone
    match first.clone() {
        Sexpr::Quote(sexpr) => Ok((*sexpr, scope.clone())),
        Sexpr::Symbol(symbol) => match symbol.as_str() {
            "lambda" => eval_rest_as_lambda(rest, scope),
            "macro" => eval_rest_as_macro_declaration(rest, scope),
            "if" => eval_rest_as_if(rest, scope),
            "let" => eval_rest_as_let(rest, scope),
            "fn" => {
                let (result, scope) = eval_rest_as_function_declaration(rest, scope)?;
                if let Sexpr::Function {
                    parameters: _,
                    body: _,
                } = &result
                {
                    let new_scope = scope.with_bindings(&[(symbol.clone(), result.clone())]);

                    Ok((result, new_scope))
                } else {
                    Err("fn must return a lambda".to_string())
                }
            }
            "quote" => match list.len() {
                2 => Ok((list[1].clone(), scope.clone())),
                _ => Err("quote must be called with one argument".to_string()),
            },
            "nil" => Err("cannot call 'nil'".to_string()),
            _ => {
                let head = first.clone().eval(scope)?.0;

                eval_list(
                    iter::once(head)
                        .chain(rest.iter().cloned())
                        .collect::<Vec<Sexpr>>(),
                    scope,
                )
            }
        },
        Sexpr::Function { parameters, body } => {
            let arguments = rest
                .iter()
                .cloned()
                .map(|arg| arg.eval(scope).map(|r| r.0))
                .collect::<Result<Vec<Sexpr>, String>>()?;

            if parameters.len() != arguments.len() {
                return Err("Function called with incorrect number of arguments".to_string());
            }

            // zip the args and params together
            let bindings = (*parameters)
                .iter()
                .cloned()
                .zip(arguments.iter().cloned())
                .collect::<Vec<(String, Sexpr)>>();

            let func_scope = scope.with_bindings(&bindings);
            sequential_eval(body.clone().to_vec(), &func_scope)
        }
        Sexpr::Macro { parameters, body } => {
            // DON'T EVALUATE THE MACRO BODY
            let arguments = rest;

            if parameters.len() != arguments.len() {
                return Err("Macro called with incorrect number of arguments".to_string());
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
            let expanded = body.clone().eval(macro_scope)?.0; // evaluate the macro
            Ok((expanded.eval(scope)?.0, scope.clone())) // evaluate the result of the macro in the original scope
        }
        Sexpr::List(sexprs) => {
            // let head = sexprs[0].clone().eval(scope)?.0;
            let head = eval_list(sexprs.to_vec(), scope)?.0;

            eval_list(
                iter::once(head)
                    .chain(sexprs[1..].iter().cloned())
                    .collect::<Vec<Sexpr>>(),
                scope,
            )
        }

        Sexpr::QuasiQuotedList(_l) => {
            panic!("cannot call a quasi-quoted list");
        }

        Sexpr::BuiltIn(builtin) => {
            let arguments = rest
                .iter()
                .cloned()
                .map(|arg| arg.eval(scope).map(|r| r.0))
                .collect::<Result<Vec<Sexpr>, String>>()?;
            Ok((builtin.eval(&arguments)?, scope.clone()))
        }

        // Error cases
        Sexpr::CommaUnquote(_) => Err("CommaUnquote in wrong context".to_string()),
        Sexpr::String(_) => Err("Cannot call string value".to_string()),
        Sexpr::Bool(_) => Err("Cannot call boolean value".to_string()),
        Sexpr::Int(_) => Err("Cannot call int value".to_string()),
        Sexpr::Float(_) => Err("Cannot call float value".to_string()),
        Sexpr::Nil => Err("Cannot call nil".to_string()),
    }
}

fn eval_rest_as_function_declaration(
    rest: &[Sexpr],
    scope: &Scope,
) -> Result<(Sexpr, Scope), String> {
    match &rest[0] {
        Sexpr::List(sexprs) => {
            // (<function_name> <arg1> <arg2>)
            if let Sexpr::Symbol(func_name) = &sexprs[0] {
                let arg_names = sexprs[1..]
                    .iter()
                    .map(|expr| match expr {
                        Sexpr::Symbol(name) => Ok(name.clone()),
                        _ => Err("Function arguments must be identifiers".to_string()),
                    })
                    .collect::<Result<Vec<String>, String>>()?;

                let function = Sexpr::Function {
                    parameters: arg_names,
                    body: rest[1..].to_vec(),
                };

                let new_scope = scope.with_bindings(&[(func_name.clone(), function.clone())]);
                Ok((function, new_scope.clone()))
            } else {
                Err("Function declaration must start with a symbol".to_string())
            }
        }
        _ => Err("Function declaration must have a list of arguments".to_string()),
    }
}

fn eval_rest_as_lambda(
    rest: &[Sexpr],
    scope: &Scope,
) -> Result<(Sexpr, Scope), String> {
    let args = parse_as_args(&rest[0])?;
    let fn_body = rest[1..].to_vec();

    Ok((
        Sexpr::Function {
            parameters: args,
            body: fn_body,
        },
        // this handles closures easily cos no mutation
        scope.clone(),
    ))
}

fn eval_rest_as_macro_declaration(
    rest: &[Sexpr],
    scope: &Scope,
) -> Result<(Sexpr, Scope), String> {
    let args = parse_as_args(&rest[0])?;
    let fn_body = &rest[1];

    Ok((
        Sexpr::Macro {
            parameters: args,
            body: Box::new(fn_body.clone()),
        },
        scope.clone(),
    ))
}

fn eval_rest_as_let(
    rest: &[Sexpr],
    scope: &Scope,
) -> Result<(Sexpr, Scope), String> {
    let binding_exprs = rest[..rest.len() - 1].to_vec();
    let expr = rest
        .last()
        .ok_or("let must have at least one argument".to_string())?;
    let bindings = generate_let_bindings(binding_exprs, scope)?;
    expr.clone().eval(&scope.with_bindings(&bindings))
}

fn eval_rest_as_if(
    rest: &[Sexpr],
    scope: &Scope,
) -> Result<(Sexpr, Scope), String> {
    if rest.len() != 3 {
        return Err("malformed if statement: Must have 3 arguments".to_string());
    }
    let condition = rest[0].clone();
    let if_body = rest[1].clone();
    let else_body = rest[2].clone();

    if let Sexpr::Bool(cond) = condition.eval(scope)?.0 {
        (if cond { if_body } else { else_body }).eval(scope)
    } else {
        Err("If condition must be a boolean".to_string())
    }
}

fn generate_let_bindings(
    list: Vec<Sexpr>,
    scope: &Scope,
) -> Result<Vec<(String, Sexpr)>, String> {
    list.iter()
        .cloned()
        .map(|node| match node {
            Sexpr::List(sexprs) => {
                if sexprs.len() != 2 {
                    return Err("let binding must be a list of two elements".to_string());
                }
                if let Sexpr::Symbol(ident) = &sexprs[0] {
                    let val = sexprs[1].clone().eval(scope)?.0;
                    Ok((ident.clone(), val.clone()))
                } else {
                    Err("left side of let binding must be an identifier".to_string())
                }
            }
            _ => Err("All bindings must be lists".to_string()),
        })
        .collect::<Result<Vec<(String, Sexpr)>, String>>()
}

fn parse_as_args(expr: &Sexpr) -> Result<Vec<String>, String> {
    match expr {
        Sexpr::List(sexprs) => sexprs
            .iter()
            .map(|e| match e {
                Sexpr::Symbol(ident) => Ok(ident.clone()),
                _ => Err("Function arguments must be identifiers".to_string()),
            })
            .collect::<Result<Vec<String>, String>>(),
        _ => Err("Function arguments must be a list".to_string()),
    }
}

fn sequential_eval(
    list: Vec<Sexpr>,
    scope: &Scope,
) -> Result<(Sexpr, Scope), String> {
    list.into_iter().fold(
        Ok((Sexpr::Nil, scope.clone())),
        |acc, item| {
            acc.and_then(|(_res, mut new_scope)| {
                let (evaluated, updated_scope) = item.eval(&new_scope)?;
                new_scope = updated_scope;
                Ok((evaluated, new_scope))
            })
        },
    )
}

/// for now, assume that the AST is a single SExpr
/// and just evaluate it.
/// Obvious next steps are to allow for multiple SExprs (lines)
/// and to manage a global scope being passed between them.
pub fn evaluate(ast: Ast) -> Result<Sexpr, String> {
    sequential_eval(
        ast.expressions.into_iter().map(|e| e.to_sexpr()).collect(),
        &Scope::new(),
    )
    .map(|r| r.0)
}

impl Display for Sexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sexpr::List(sexprs) => write_sexpr_vec(f, sexprs),
            Sexpr::Quote(sexpr) => write!(f, "'{}", sexpr),
            Sexpr::QuasiQuotedList(sexprs) => {
                write!(f, "`")?;
                write_sexpr_vec(f, sexprs)
            }
            Sexpr::Symbol(sym) => write!(f, "{}", sym), // might want :{} later
            Sexpr::String(str) => write!(f, "\"{}\"", str),
            Sexpr::Bool(b) => write!(f, "{}", b),
            Sexpr::Int(i) => write!(f, "{}", i),
            Sexpr::Float(fl) => write!(f, "{}", fl),
            Sexpr::Function {
                parameters: _,
                body: _,
            } => write!(f, "Function"),
            Sexpr::Macro {
                parameters: _,
                body: _,
            } => write!(f, "Macro"),
            Sexpr::BuiltIn(b) => write!(f, "<builtin: {}>", b.symbol),
            Sexpr::CommaUnquote(sexpr) => write!(f, ",{}", sexpr),
            Sexpr::Nil => write!(f, "nil"),
        }
    }
}

fn write_sexpr_vec(
    f: &mut std::fmt::Formatter,
    sexprs: &[Sexpr],
) -> Result<(), std::fmt::Error> {
    write!(f, "(")?;
    for (i, sexpr) in sexprs.iter().enumerate() {
        write!(f, "{}", sexpr)?;
        if i < sexprs.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, ")")?;
    Ok(())
}

pub struct Session {
    scope: Scope,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Session {
        Session {
            scope: Scope::new(),
        }
    }

    /// Evaluates a single expression, mutating the session's scope
    /// and returning the result of the evaluation.
    pub fn eval(&mut self, expr: Sexpr) -> Result<Sexpr, String> {
        let (res, new_scope) = expr.eval(&self.scope)?;
        self.scope = new_scope;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<(), String> {
        let expr = Sexpr::List(vec![
            Sexpr::Symbol("+".to_string()),
            Sexpr::Int(1),
            Sexpr::Int(2),
        ]);
        let output = expr.eval(&Scope::new())?.0;
        assert_eq!(output, Sexpr::Int(3));
        Ok(())
    }

    #[test]
    fn test2() -> Result<(), String> {
        let sexpr = Sexpr::List(vec![
            Sexpr::Symbol("+".to_string()),
            Sexpr::Int(1),
            Sexpr::Int(2),
            Sexpr::List(vec![
                Sexpr::Symbol("-".to_string()),
                Sexpr::Int(4),
                Sexpr::Int(3),
            ]),
            Sexpr::Int(5),
            Sexpr::List(vec![
                Sexpr::Symbol("*".to_string()),
                Sexpr::Int(1),
                Sexpr::Int(2),
            ]),
        ]);
        let res = sexpr.eval(&Scope::new())?.0;
        assert_eq!(res, Sexpr::Int(11));
        Ok(())
    }

    #[test]
    fn test3() -> Result<(), String> {
        let sexpr = Sexpr::List(vec![
            Sexpr::Symbol("let".to_string()),
            Sexpr::List(vec![
                Sexpr::Symbol("x".to_string()),
                Sexpr::Int(2),
            ]),
            Sexpr::List(vec![
                Sexpr::Symbol("*".to_string()),
                Sexpr::Symbol("x".to_string()),
                Sexpr::Int(3),
            ]),
        ]);
        let res = sexpr.eval(&Scope::new())?.0;
        assert_eq!(res, Sexpr::Int(6));
        Ok(())
    }

    #[test]
    fn test_macros_1() -> Result<(), String> {
        /*
         * (let
         *   (switch (macro (a b) (quote (b a))))
         *   (switch 3 inc))
         */
        let ast = Sexpr::List(vec![
            Sexpr::Symbol("let".to_string()),
            Sexpr::List(vec![
                Sexpr::Symbol("switch".to_string()),
                Sexpr::List(vec![
                    Sexpr::Symbol("macro".to_string()),
                    Sexpr::List(vec![
                        Sexpr::Symbol("a".to_string()),
                        Sexpr::Symbol("b".to_string()),
                    ]),
                    Sexpr::List(vec![
                        Sexpr::Symbol("list".to_string()),
                        Sexpr::Symbol("b".to_string()),
                        Sexpr::Symbol("a".to_string()),
                    ]),
                ]),
            ]),
            Sexpr::List(vec![
                Sexpr::Symbol("switch".to_string()),
                Sexpr::Int(1),
                Sexpr::Symbol("inc".to_string()),
            ]),
        ]);
        let res = ast.eval(&Scope::new())?.0;
        assert_eq!(res, Sexpr::Int(2));
        Ok(())
    }

    #[test]
    fn test_macros_2() -> Result<(), String> {
        /*
         * (let
         *   (switch (macro (a b) (quote (b a))))
         *   (switch 3 inc))
         */
        let ast = Sexpr::List(vec![
            Sexpr::Symbol("let".to_string()),
            Sexpr::List(vec![
                Sexpr::Symbol("switch".to_string()),
                Sexpr::List(vec![
                    Sexpr::Symbol("macro".to_string()),
                    Sexpr::List(vec![
                        Sexpr::Symbol("a".to_string()),
                        Sexpr::Symbol("b".to_string()),
                    ]),
                    Sexpr::List(vec![
                        Sexpr::Symbol("list".to_string()),
                        Sexpr::Symbol("b".to_string()),
                        Sexpr::Symbol("a".to_string()),
                    ]),
                ]),
            ]),
            Sexpr::List(vec![
                Sexpr::Symbol("switch".to_string()),
                Sexpr::Int(1),
                Sexpr::Symbol("inc".to_string()),
            ]),
        ]);
        let res = ast.eval(&Scope::new())?.0;
        assert_eq!(res, Sexpr::Int(2));
        Ok(())
    }

    #[test]
    fn test_macros_3() -> Result<(), String> {
        /*
         * (let
         *   (infix (macro (a op b) (list op a b)))
         *   (infix 1 + 2))
         */
        let ast = Sexpr::List(vec![
            Sexpr::Symbol("let".to_string()),
            Sexpr::List(vec![
                Sexpr::Symbol("infix".to_string()),
                Sexpr::List(vec![
                    Sexpr::Symbol("macro".to_string()),
                    Sexpr::List(vec![
                        Sexpr::Symbol("a".to_string()),
                        Sexpr::Symbol("op".to_string()),
                        Sexpr::Symbol("b".to_string()),
                    ]),
                    Sexpr::List(vec![
                        Sexpr::Symbol("list".to_string()),
                        Sexpr::Symbol("op".to_string()),
                        Sexpr::Symbol("a".to_string()),
                        Sexpr::Symbol("b".to_string()),
                    ]),
                ]),
            ]),
            Sexpr::List(vec![
                Sexpr::Symbol("infix".to_string()),
                Sexpr::Int(1),
                Sexpr::Symbol("+".to_string()),
                Sexpr::Int(2),
            ]),
        ]);
        assert_eq!(ast.eval(&Scope::new())?.0, Sexpr::Int(3));
        Ok(())
    }
}
