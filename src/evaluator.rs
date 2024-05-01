use std::collections::HashMap;
use std::fmt::Display;
use std::iter;

use crate::builtins::BUILT_INS;
use crate::parser::Ast;
use crate::sexpr::EvauluatorRuntimeValue;

#[derive(Debug, Clone, PartialEq)]
struct Scope {
    bindings: HashMap<String, EvauluatorRuntimeValue>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            bindings: HashMap::from_iter(BUILT_INS.map(|builtin| {
                (
                    builtin.symbol.to_string(),
                    EvauluatorRuntimeValue::BuiltIn(builtin),
                )
            })),
        }
    }

    fn with_bindings(&self, bindings: &[(String, EvauluatorRuntimeValue)]) -> Scope {
        let mut new_bindings = self.bindings.clone();
        new_bindings.extend(bindings.iter().cloned());

        Scope {
            bindings: new_bindings,
        }
    }
}

impl EvauluatorRuntimeValue {
    fn eval(self, scope: &Scope) -> Result<(EvauluatorRuntimeValue, Scope), String> {
        match self {
            EvauluatorRuntimeValue::List(sexprs) => eval_list(sexprs, scope), // todo remove this arg
            EvauluatorRuntimeValue::QuasiQuotedList(sexprs) => {
                let res = sexprs
                    .iter()
                    .map(|sexpr| match sexpr {
                        EvauluatorRuntimeValue::CommaUnquote(sexpr) => {
                            sexpr.clone().eval(scope).map(|r| r.0)
                        }
                        sexpr => Ok(sexpr.clone()),
                    })
                    .collect::<Result<Vec<EvauluatorRuntimeValue>, String>>()?;

                Ok((EvauluatorRuntimeValue::List(res), scope.clone()))
            }
            EvauluatorRuntimeValue::Quote(sexpr) => Ok((*sexpr, scope.clone())),
            EvauluatorRuntimeValue::Symbol(sym) => match sym.as_str() {
                "nil" => Ok((EvauluatorRuntimeValue::Nil, scope.clone())),
                _ => match scope.bindings.get(&sym) {
                    Some(sexpr) => Ok((sexpr.clone(), scope.clone())),
                    None => Err(format!("Symbol {} not found in scope", sym)),
                },
            },
            // self-evaluating S-expressions
            EvauluatorRuntimeValue::String(_)
            | EvauluatorRuntimeValue::Bool(_)
            | EvauluatorRuntimeValue::Int(_)
            | EvauluatorRuntimeValue::Float(_)
            | EvauluatorRuntimeValue::BuiltIn(_)
            | EvauluatorRuntimeValue::CommaUnquote(_)
            | EvauluatorRuntimeValue::Macro {
                parameters: _,
                body: _,
            }
            | EvauluatorRuntimeValue::Function {
                parameters: _,
                body: _,
            }
            | EvauluatorRuntimeValue::Nil => Ok((self, scope.clone())),
        }
    }
}

fn eval_list(
    list: Vec<EvauluatorRuntimeValue>,
    scope: &Scope,
) -> Result<(EvauluatorRuntimeValue, Scope), String> {
    if list.is_empty() {
        return Ok((EvauluatorRuntimeValue::List(vec![]), scope.clone()));
    }

    let (first, rest) = list.split_first().unwrap();

    // FIXME fix this clone
    match first.clone() {
        EvauluatorRuntimeValue::Quote(sexpr) => Ok((*sexpr, scope.clone())),
        EvauluatorRuntimeValue::Symbol(symbol) => match symbol.as_str() {
            "lambda" => eval_rest_as_lambda(rest, scope),
            "macro" => eval_rest_as_macro_declaration(rest, scope),
            "if" => eval_rest_as_if(rest, scope),
            "let" => eval_rest_as_let(rest, scope),
            "fn" => {
                let (result, scope) = eval_rest_as_function_declaration(rest, scope)?;
                if let EvauluatorRuntimeValue::Function {
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
                        .collect::<Vec<EvauluatorRuntimeValue>>(),
                    scope,
                )
            }
        },
        EvauluatorRuntimeValue::Function { parameters, body } => {
            let arguments = rest
                .iter()
                .cloned()
                .map(|arg| arg.eval(scope).map(|r| r.0))
                .collect::<Result<Vec<EvauluatorRuntimeValue>, String>>()?;

            if parameters.len() != arguments.len() {
                return Err("Function called with incorrect number of arguments".to_string());
            }

            // zip the args and params together
            let bindings = (*parameters)
                .iter()
                .cloned()
                .zip(arguments.iter().cloned())
                .collect::<Vec<(String, EvauluatorRuntimeValue)>>();

            let func_scope = scope.with_bindings(&bindings);
            sequential_eval(body.clone().to_vec(), &func_scope)
        }
        EvauluatorRuntimeValue::Macro { parameters, body } => {
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
                .collect::<Vec<(String, EvauluatorRuntimeValue)>>();

            // create a new scope with the macro_bindings for inside the macro
            let macro_scope = &scope.with_bindings(&macro_bindings);
            let expanded = body.clone().eval(macro_scope)?.0; // evaluate the macro
            Ok((expanded.eval(scope)?.0, scope.clone())) // evaluate the result of the macro in the original scope
        }
        EvauluatorRuntimeValue::List(sexprs) => {
            // let head = sexprs[0].clone().eval(scope)?.0;
            let head = eval_list(sexprs.to_vec(), scope)?.0;

            eval_list(
                iter::once(head)
                    .chain(sexprs[1..].iter().cloned())
                    .collect::<Vec<EvauluatorRuntimeValue>>(),
                scope,
            )
        }

        EvauluatorRuntimeValue::QuasiQuotedList(_l) => {
            panic!("cannot call a quasi-quoted list");
        }

        EvauluatorRuntimeValue::BuiltIn(builtin) => {
            let arguments = rest
                .iter()
                .cloned()
                .map(|arg| arg.eval(scope).map(|r| r.0))
                .collect::<Result<Vec<EvauluatorRuntimeValue>, String>>()?;
            Ok((builtin.eval(&arguments)?, scope.clone()))
        }

        // Error cases
        EvauluatorRuntimeValue::CommaUnquote(_) => Err("CommaUnquote in wrong context".to_string()),
        EvauluatorRuntimeValue::String(_) => Err("Cannot call string value".to_string()),
        EvauluatorRuntimeValue::Bool(_) => Err("Cannot call boolean value".to_string()),
        EvauluatorRuntimeValue::Int(_) => Err("Cannot call int value".to_string()),
        EvauluatorRuntimeValue::Float(_) => Err("Cannot call float value".to_string()),
        EvauluatorRuntimeValue::Nil => Err("Cannot call nil".to_string()),
    }
}

fn eval_rest_as_function_declaration(
    rest: &[EvauluatorRuntimeValue],
    scope: &Scope,
) -> Result<(EvauluatorRuntimeValue, Scope), String> {
    match &rest[0] {
        EvauluatorRuntimeValue::List(sexprs) => {
            // (<function_name> <arg1> <arg2>)
            if let EvauluatorRuntimeValue::Symbol(func_name) = &sexprs[0] {
                let arg_names = sexprs[1..]
                    .iter()
                    .map(|expr| match expr {
                        EvauluatorRuntimeValue::Symbol(name) => Ok(name.clone()),
                        _ => Err("Function arguments must be identifiers".to_string()),
                    })
                    .collect::<Result<Vec<String>, String>>()?;

                let function = EvauluatorRuntimeValue::Function {
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
    rest: &[EvauluatorRuntimeValue],
    scope: &Scope,
) -> Result<(EvauluatorRuntimeValue, Scope), String> {
    let args = parse_as_args(&rest[0])?;
    let fn_body = rest[1..].to_vec();

    // TODO closures ???
    // substitute scope into fn_body ???
    // actually should be easy as everything is pure and passed by value
    let _ = scope;

    Ok((
        EvauluatorRuntimeValue::Function {
            parameters: args,
            body: fn_body,
        },
        scope.clone(),
    ))
}

fn eval_rest_as_macro_declaration(
    rest: &[EvauluatorRuntimeValue],
    scope: &Scope,
) -> Result<(EvauluatorRuntimeValue, Scope), String> {
    let args = parse_as_args(&rest[0])?;
    let fn_body = &rest[1];

    // todo substitute scope into fn_body
    let _ = scope;

    Ok((
        EvauluatorRuntimeValue::Macro {
            parameters: args,
            body: Box::new(fn_body.clone()),
        },
        scope.clone(),
    ))
}

fn eval_rest_as_let(
    rest: &[EvauluatorRuntimeValue],
    scope: &Scope,
) -> Result<(EvauluatorRuntimeValue, Scope), String> {
    let binding_exprs = rest[..rest.len() - 1].to_vec();
    let expr = rest
        .last()
        .ok_or("let must have at least one argument".to_string())?;
    let bindings = generate_let_bindings(binding_exprs, scope)?;
    expr.clone().eval(&scope.with_bindings(&bindings))
}

fn eval_rest_as_if(
    rest: &[EvauluatorRuntimeValue],
    scope: &Scope,
) -> Result<(EvauluatorRuntimeValue, Scope), String> {
    if rest.len() != 3 {
        return Err("malformed if statement: Must have 3 arguments".to_string());
    }
    let condition = rest[0].clone();
    let if_body = rest[1].clone();
    let else_body = rest[2].clone();

    // TODO: encapse this is Sexpr.bool()
    if let EvauluatorRuntimeValue::Bool(cond) = condition.eval(scope)?.0 {
        (if cond { if_body } else { else_body }).eval(scope)
    } else {
        Err("If condition must be a boolean".to_string())
    }
}

fn generate_let_bindings(
    list: Vec<EvauluatorRuntimeValue>,
    scope: &Scope,
) -> Result<Vec<(String, EvauluatorRuntimeValue)>, String> {
    list.iter()
        .cloned()
        .map(|node| match node {
            EvauluatorRuntimeValue::List(sexprs) => {
                if sexprs.len() != 2 {
                    return Err("let binding must be a list of two elements".to_string());
                }
                if let EvauluatorRuntimeValue::Symbol(ident) = &sexprs[0] {
                    let val = sexprs[1].clone().eval(scope)?.0;
                    Ok((ident.clone(), val.clone()))
                } else {
                    Err("left side of let binding must be an identifier".to_string())
                }
            }
            _ => Err("All bindings must be lists".to_string()),
        })
        .collect::<Result<Vec<(String, EvauluatorRuntimeValue)>, String>>()
}

fn parse_as_args(expr: &EvauluatorRuntimeValue) -> Result<Vec<String>, String> {
    match expr {
        EvauluatorRuntimeValue::List(sexprs) => sexprs
            .iter()
            .map(|e| match e {
                EvauluatorRuntimeValue::Symbol(ident) => Ok(ident.clone()),
                _ => Err("Function arguments must be identifiers".to_string()),
            })
            .collect::<Result<Vec<String>, String>>(),
        _ => Err("Function arguments must be a list".to_string()),
    }
}

fn sequential_eval(
    list: Vec<EvauluatorRuntimeValue>,
    scope: &Scope,
) -> Result<(EvauluatorRuntimeValue, Scope), String> {
    list.into_iter().fold(
        Ok((EvauluatorRuntimeValue::Nil, scope.clone())),
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
pub fn evaluate(ast: Ast) -> Result<EvauluatorRuntimeValue, String> {
    sequential_eval(
        ast.expressions.into_iter().map(|e| e.to_sexpr()).collect(),
        &Scope::new(),
    )
    .map(|r| r.0)
}

impl Display for EvauluatorRuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvauluatorRuntimeValue::List(sexprs) => write_sexpr_vec(f, sexprs),
            EvauluatorRuntimeValue::Quote(sexpr) => write!(f, "'{}", sexpr),
            EvauluatorRuntimeValue::QuasiQuotedList(sexprs) => {
                write!(f, "`")?;
                write_sexpr_vec(f, sexprs)
            }
            EvauluatorRuntimeValue::Symbol(sym) => write!(f, "{}", sym), // might want :{} later
            EvauluatorRuntimeValue::String(str) => write!(f, "\"{}\"", str),
            EvauluatorRuntimeValue::Bool(b) => write!(f, "{}", b),
            EvauluatorRuntimeValue::Int(i) => write!(f, "{}", i),
            EvauluatorRuntimeValue::Float(fl) => write!(f, "{}", fl),
            EvauluatorRuntimeValue::Function {
                parameters: _,
                body: _,
            } => write!(f, "Function"),
            EvauluatorRuntimeValue::Macro {
                parameters: _,
                body: _,
            } => write!(f, "Macro"),
            EvauluatorRuntimeValue::BuiltIn(b) => write!(f, "<builtin: {}>", b.symbol),
            EvauluatorRuntimeValue::CommaUnquote(sexpr) => write!(f, ",{}", sexpr),
            EvauluatorRuntimeValue::Nil => write!(f, "nil"),
        }
    }
}

fn write_sexpr_vec(
    f: &mut std::fmt::Formatter,
    sexprs: &[EvauluatorRuntimeValue],
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
    pub fn eval(&mut self, expr: EvauluatorRuntimeValue) -> Result<EvauluatorRuntimeValue, String> {
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
        let expr = EvauluatorRuntimeValue::List(vec![
            EvauluatorRuntimeValue::Symbol("+".to_string()),
            EvauluatorRuntimeValue::Int(1),
            EvauluatorRuntimeValue::Int(2),
        ]);
        let output = expr.eval(&Scope::new())?.0;
        assert_eq!(output, EvauluatorRuntimeValue::Int(3));
        Ok(())
    }

    #[test]
    fn test2() -> Result<(), String> {
        let sexpr = EvauluatorRuntimeValue::List(vec![
            EvauluatorRuntimeValue::Symbol("+".to_string()),
            EvauluatorRuntimeValue::Int(1),
            EvauluatorRuntimeValue::Int(2),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("-".to_string()),
                EvauluatorRuntimeValue::Int(4),
                EvauluatorRuntimeValue::Int(3),
            ]),
            EvauluatorRuntimeValue::Int(5),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("*".to_string()),
                EvauluatorRuntimeValue::Int(1),
                EvauluatorRuntimeValue::Int(2),
            ]),
        ]);
        let res = sexpr.eval(&Scope::new())?.0;
        assert_eq!(res, EvauluatorRuntimeValue::Int(11));
        Ok(())
    }

    #[test]
    fn test3() -> Result<(), String> {
        let sexpr = EvauluatorRuntimeValue::List(vec![
            EvauluatorRuntimeValue::Symbol("let".to_string()),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("x".to_string()),
                EvauluatorRuntimeValue::Int(2),
            ]),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("*".to_string()),
                EvauluatorRuntimeValue::Symbol("x".to_string()),
                EvauluatorRuntimeValue::Int(3),
            ]),
        ]);
        let res = sexpr.eval(&Scope::new())?.0;
        assert_eq!(res, EvauluatorRuntimeValue::Int(6));
        Ok(())
    }

    #[test]
    fn test_macros_1() -> Result<(), String> {
        /*
         * (let
         *   (switch (macro (a b) (quote (b a))))
         *   (switch 3 inc))
         */
        let ast = EvauluatorRuntimeValue::List(vec![
            EvauluatorRuntimeValue::Symbol("let".to_string()),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("switch".to_string()),
                EvauluatorRuntimeValue::List(vec![
                    EvauluatorRuntimeValue::Symbol("macro".to_string()),
                    EvauluatorRuntimeValue::List(vec![
                        EvauluatorRuntimeValue::Symbol("a".to_string()),
                        EvauluatorRuntimeValue::Symbol("b".to_string()),
                    ]),
                    EvauluatorRuntimeValue::List(vec![
                        EvauluatorRuntimeValue::Symbol("list".to_string()),
                        EvauluatorRuntimeValue::Symbol("b".to_string()),
                        EvauluatorRuntimeValue::Symbol("a".to_string()),
                    ]),
                ]),
            ]),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("switch".to_string()),
                EvauluatorRuntimeValue::Int(1),
                EvauluatorRuntimeValue::Symbol("inc".to_string()),
            ]),
        ]);
        let res = ast.eval(&Scope::new())?.0;
        assert_eq!(res, EvauluatorRuntimeValue::Int(2));
        Ok(())
    }

    #[test]
    fn test_macros_2() -> Result<(), String> {
        /*
         * (let
         *   (switch (macro (a b) (quote (b a))))
         *   (switch 3 inc))
         */
        let ast = EvauluatorRuntimeValue::List(vec![
            EvauluatorRuntimeValue::Symbol("let".to_string()),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("switch".to_string()),
                EvauluatorRuntimeValue::List(vec![
                    EvauluatorRuntimeValue::Symbol("macro".to_string()),
                    EvauluatorRuntimeValue::List(vec![
                        EvauluatorRuntimeValue::Symbol("a".to_string()),
                        EvauluatorRuntimeValue::Symbol("b".to_string()),
                    ]),
                    EvauluatorRuntimeValue::List(vec![
                        EvauluatorRuntimeValue::Symbol("list".to_string()),
                        EvauluatorRuntimeValue::Symbol("b".to_string()),
                        EvauluatorRuntimeValue::Symbol("a".to_string()),
                    ]),
                ]),
            ]),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("switch".to_string()),
                EvauluatorRuntimeValue::Int(1),
                EvauluatorRuntimeValue::Symbol("inc".to_string()),
            ]),
        ]);
        let res = ast.eval(&Scope::new())?.0;
        assert_eq!(res, EvauluatorRuntimeValue::Int(2));
        Ok(())
    }

    #[test]
    fn test_macros_3() -> Result<(), String> {
        /*
         * (let
         *   (infix (macro (a op b) (list op a b)))
         *   (infix 1 + 2))
         */
        let ast = EvauluatorRuntimeValue::List(vec![
            EvauluatorRuntimeValue::Symbol("let".to_string()),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("infix".to_string()),
                EvauluatorRuntimeValue::List(vec![
                    EvauluatorRuntimeValue::Symbol("macro".to_string()),
                    EvauluatorRuntimeValue::List(vec![
                        EvauluatorRuntimeValue::Symbol("a".to_string()),
                        EvauluatorRuntimeValue::Symbol("op".to_string()),
                        EvauluatorRuntimeValue::Symbol("b".to_string()),
                    ]),
                    EvauluatorRuntimeValue::List(vec![
                        EvauluatorRuntimeValue::Symbol("list".to_string()),
                        EvauluatorRuntimeValue::Symbol("op".to_string()),
                        EvauluatorRuntimeValue::Symbol("a".to_string()),
                        EvauluatorRuntimeValue::Symbol("b".to_string()),
                    ]),
                ]),
            ]),
            EvauluatorRuntimeValue::List(vec![
                EvauluatorRuntimeValue::Symbol("infix".to_string()),
                EvauluatorRuntimeValue::Int(1),
                EvauluatorRuntimeValue::Symbol("+".to_string()),
                EvauluatorRuntimeValue::Int(2),
            ]),
        ]);
        assert_eq!(ast.eval(&Scope::new())?.0, EvauluatorRuntimeValue::Int(3));
        Ok(())
    }
}
