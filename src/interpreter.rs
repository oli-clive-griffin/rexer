// use std::collections::HashMap;

use crate::parser::{AST, SExpr};
use crate::lexer::{Literal, NumericLiteral, Operator};
use crate::runtime_value::RuntimeValue;

// struct Scope {
//     // could make this a list of hashmaps that's search from the top down
//     // would negate the need to duplicate the scope when adding items
//     bindings: HashMap<String, RuntimeValue>,
// }

// impl Scope {
//     fn new () -> Scope {
//         Scope { bindings: HashMap::new() }
//     }
// }

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
    fn eval(&self, /*scope: &Scope*/) -> RuntimeValue {
        match self {
            SExpr::List(list) => {
                let first_sexpr = &list[0];

                if let SExpr::Op(op) = first_sexpr { // for now only handle operators
                    let rest_val = &list[1..].iter().map(|e| e.eval(/*scope*/)).collect::<Vec<RuntimeValue>>();
                    return op.execute(rest_val);
                } else {
                    panic!("First element of list must be an operator")
                }
            }
            SExpr::Literal(lit) => RuntimeValue::from_literal(lit),
            SExpr::Boolean(bool) => RuntimeValue::Boolean(*bool),
            SExpr::Op(op) => panic!("Cannot evaluate operator {:?}", op), // remove this eventually, this is simple to handle basic math
            SExpr::Ident(ident) => {
                todo!()
                // match scope.bindings.get(ident) {
                //     Some(val) => val.clone(), // inefficient, but for now just clone the value
                //     None => panic!("Identifier {} not found in scope", ident),
                // }
            }
        }
    }
}

/// for now, assume that the AST is a single SExpr
/// and just evaluate it
pub fn interpret(ast: &AST) {
    println!("AST: {:#?}", &ast);

    return match ast.root.eval(/*&Scope::new()*/) {
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

        let output = ast.root.eval(/*&Scope::new()*/);

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
            ast.root.eval(/*&Scope::new()*/),
            RuntimeValue::Float(11.3),
        )
    }
}
