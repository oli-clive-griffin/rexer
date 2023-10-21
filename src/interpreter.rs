use std::collections::HashMap;

use crate::parser::{AST, SExpr, parse};
use crate::lexer::{Literal, NumericLiteral, Operator};
use crate::runtime_value::{RuntimeValue, Function};

struct Scope {
    // could make this a list of hashmaps that's search from the top down
    // would negate the need to duplicate the scope when adding items
    bindings: HashMap<String, RuntimeValue>,
}

impl Scope {
    fn new () -> Scope {
        Scope { bindings: HashMap::new() }
    }

    fn with_bindings(&self, bindings: Vec<(String, RuntimeValue)>) -> Scope {
        let mut new_bindings = self.bindings.clone();
        for (ident, val) in bindings {
            new_bindings.insert(ident, val);
        }
        Scope { bindings: new_bindings }
    }
}

impl RuntimeValue {
    fn from_literal(lit: &Literal) -> RuntimeValue {
        match lit {
            Literal::Numeric(n) => match n {
                NumericLiteral::Int(i) => RuntimeValue::Int(*i),
                NumericLiteral::Float(f) => RuntimeValue::Float(*f),
            }
            Literal::String(s) => RuntimeValue::String(s.clone()),
        }
    }
}

impl Operator {
    fn execute(&self, args: &[RuntimeValue]) -> RuntimeValue {
        args
            .iter()
            .cloned()
            .reduce(|acc, val| { self.binary(acc, val) })// cannot return reference to temporary value returns a reference to data owned by the current functionrustcClick for full compiler diagnostic. temporary value created here
            .unwrap()
    }

    fn binary(&self, a: RuntimeValue, b: RuntimeValue) -> RuntimeValue {
        match self {
            Operator::Plus => a + b,
            Operator::Divide => a / b,
            Operator::Multiply => a * b,
            Operator::Minus => a - b,
        }
    }
}

impl SExpr {
    fn eval(&self, scope: &Scope) -> RuntimeValue {
        match self {
            SExpr::List(list) => eval_list(list, scope),
            SExpr::Literal(lit) => RuntimeValue::from_literal(lit),
            SExpr::Boolean(bool) => RuntimeValue::Boolean(*bool),
            SExpr::Op(op) => panic!("Cannot evaluate operator {:?}", op), // remove this eventually, this is simple to handle basic math
            SExpr::Indent(ident) => {
                match scope.bindings.get(ident) {
                    Some(val) => val.clone(), // inefficient, but for now just clone the value
                    None => panic!("Identifier {} not found in scope", ident),
                }
            }
            SExpr::Fn => panic!(),
            SExpr::If => todo!(),
            SExpr::Let => todo!(),
            
        }
    }
}

fn eval_list(list: &Vec<SExpr>, scope: &Scope) -> RuntimeValue {
    let first_sexpr = &list[0];

    match first_sexpr {
        SExpr::Fn => {
            let args = parse_as_args(&list[1]);

            let fn_body = &list[2];

            return RuntimeValue::Function(Function {
                params: args,
                body: fn_body.clone(),
            });
        }
        SExpr::If => {
            let condition = &list[1];
            let if_body = &list[2];
            let else_body = &list[3];

            if condition.eval(scope).bool() {
                return if_body.eval(scope);
            } else {
                return else_body.eval(scope);
            }
        }
        SExpr::Let => {
            let binding_exprs = &list[1..list.len()-2]; // ignore both the let and the expr
            let bindings = generate_bindings(binding_exprs, scope);
            let expr = &list[list.len()-1];

            return expr.eval(&scope.with_bindings(bindings));
        }
        SExpr::Indent(ident) => {
            let head_val = scope.bindings.get(ident).unwrap();
            match head_val {
                RuntimeValue::Function(func) => {
                    return evaluation_function(func, &list[1..], scope);
                }
                _ => panic!("Cannot call non-function value"),
            }
        }
        SExpr::Op(op) => { // for now only handle operators
            let rest_val = &list[1..].iter().map(|e| e.eval(scope)).collect::<Vec<RuntimeValue>>();
            return op.execute(rest_val);
        }
        SExpr::List(_) => todo!(),
        SExpr::Literal(_)
        | SExpr::Boolean(_) => panic!(),
    
    };
}

fn evaluation_function(func: &Function, list: &[SExpr], scope: &Scope) -> RuntimeValue {
    todo!()
}

fn generate_bindings(list: &[SExpr], scope: &Scope) -> Vec<(String, RuntimeValue)> {
    list
        .iter()
        .cloned()
        .map(|node| {
            match node {
                SExpr::List(nodes) => {
                    if nodes.len() != 2 {
                        panic!("let binding must be a list of two elements");
                    }
                    if let SExpr::Indent(ident) = &nodes[0] {
                        let val = &nodes[1].eval(scope);
                        return (ident.clone(), val.clone());
                    } else {
                        panic!("left side of let binding must be an identifier");
                    }
                }
                _ => panic!("All bindings must be lists")
            }
        })
        .collect::<Vec<(String, RuntimeValue)>>()
}

fn parse_as_args(expr: &SExpr) -> Vec<String> {
    if let SExpr::List(args) = expr {
        args.iter().map(|e| {
            if let SExpr::Indent(ident) = e {
                ident.clone()
            } else {
                panic!("Function arguments must be identifiers")
            }
        }).collect::<Vec<String>>()
    } else {
        panic!("Function arguments must be a list")
    }
}

// fn eval_function

/// for now, assume that the AST is a single SExpr
/// and just evaluate it
pub fn interpret(ast: &AST) {
    println!("AST: {:#?}", &ast);

    return match ast.root.eval(&Scope::new()) {
        RuntimeValue::Boolean(b) => println!("Boolean: {}", b),
        RuntimeValue::Float(f) => println!("Float: {}", f),
        RuntimeValue::Int(i) => println!("Int: {}", i),
        RuntimeValue::String(s) => println!("String: {}", s),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let ast = AST {
            root: SExpr::List(vec![
                SExpr::Op(Operator::Plus),
                SExpr::Literal(Literal::Numeric(NumericLiteral::Int(1))),
                SExpr::Literal(Literal::Numeric(NumericLiteral::Int(2))),
            ])
        };

        let output = ast.root.eval(&Scope::new());

        assert_eq!(output, RuntimeValue::Int(3));
    }

    #[test]
    fn test2() {
        let ast = AST {
            root: SExpr::List(vec![
                SExpr::Op(Operator::Plus),
                SExpr::Literal(Literal::Numeric(NumericLiteral::Int(1))),
                SExpr::Literal(Literal::Numeric(NumericLiteral::Int(2))),
                SExpr::List(vec![
                    SExpr::Op(Operator::Minus),
                    SExpr::Literal(Literal::Numeric(NumericLiteral::Int(4))),
                    SExpr::Literal(Literal::Numeric(NumericLiteral::Int(3))),
                ]),
                SExpr::Literal(Literal::Numeric(NumericLiteral::Int(5))),
                SExpr::List(vec![
                    SExpr::Op(Operator::Multiply),
                    SExpr::Literal(Literal::Numeric(NumericLiteral::Int(1))),
                    SExpr::Literal(Literal::Numeric(NumericLiteral::Float(2.3))),
                ]),
            ])
        };

        interpret(&ast);

        assert_eq!(
            ast.root.eval(&Scope::new()),
            RuntimeValue::Float(11.3),
        )
    }
}
