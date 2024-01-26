use core::panic;
use std::collections::HashMap;

use crate::parser::{Literal, Node, NumericLiteral, Operator, AST};
use crate::runtime_value::{Function, RuntimeValue};

struct Scope {
    // could make this a list of hashmaps that's search from the top down
    // would negate the need to duplicate the scope when adding items
    bindings: HashMap<String, RuntimeValue>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            bindings: HashMap::new(),
        }
    }

    fn with_bindings(&self, bindings: Vec<(String, RuntimeValue)>) -> Scope {
        let mut new_bindings = self.bindings.clone();
        new_bindings.extend(bindings);

        Scope {
            bindings: new_bindings,
        }
    }
}

impl RuntimeValue {
    fn from_literal(lit: &Literal) -> RuntimeValue {
        match lit {
            Literal::Numeric(n) => match n {
                NumericLiteral::Int(i) => RuntimeValue::Int(*i),
                NumericLiteral::Float(f) => RuntimeValue::Float(*f),
            },
            Literal::String(s) => RuntimeValue::String(s.clone()),
            Literal::Boolean(b) => RuntimeValue::Boolean(*b), // cloned
        }
    }
}

impl Operator {
    fn execute(&self, args: &[RuntimeValue]) -> RuntimeValue {
        args.iter()
            .cloned()
            .reduce(|acc, val| self.binary(acc, val)) // cannot return reference to temporary value returns a reference to data owned by the current functionrustcClick for full compiler diagnostic. temporary value created here
            .unwrap()
    }

    fn binary(&self, a: RuntimeValue, b: RuntimeValue) -> RuntimeValue {
        match self {
            Operator::Add => a + b,
            Operator::Div => a / b,
            Operator::Mul => a * b,
            Operator::Sub => a - b,
        }
    }
}

impl Node {
    fn eval(&self, scope: &Scope) -> RuntimeValue {
        match self {
            Node::List(list) => eval_list(list, scope),
            Node::Literal(lit) => RuntimeValue::from_literal(lit),
            Node::Op(op) => RuntimeValue::Op(*op),
            Node::Ident(ident) => scope
                .bindings
                .get(ident)
                .expect(format!("Identifier {ident} not found in scope").as_str())
                .clone(),
            thing => panic!("should not be parsed as a node: {:?}", thing),
        }
    }
}

fn eval_list(list: &Vec<Node>, scope: &Scope) -> RuntimeValue {
    // I think this kind of pattern matching is a code smell,
    // there should be a "special form" type that evaluates itself
    match list[0] {
        // handle special forms:
        Node::Fn => return eval_list_as_function_declaration(list, scope),
        Node::If => return eval_if(list, scope),
        Node::Let => return eval_let(list, scope),
        Node::Quote => return eval_quote(list, scope),
        _ => (),
    }

    let vals = list
        .iter()
        .map(|arg| arg.eval(scope))
        .collect::<Vec<RuntimeValue>>();

    let args_vals = &vals[1..];
    let head_val = &vals[0];

    match head_val {
        RuntimeValue::Function(func) => eval_fun(func, args_vals, scope),
        RuntimeValue::Op(op) => op.execute(args_vals),
        RuntimeValue::Int(_) => panic!("Cannot call int value"),
        RuntimeValue::List(_) => panic!("Cannot call list value"),
        RuntimeValue::Float(_) => panic!("Cannot call float value"),
        RuntimeValue::String(_) => panic!("Cannot call string value"),
        RuntimeValue::Boolean(_) => panic!("Cannot call boolean value"),
    }
}

fn eval_quote(list: &Vec<Node>, scope: &Scope) -> RuntimeValue {
    RuntimeValue::List(list[1..].iter().map(|arg| arg.eval(scope)).collect())
}

fn eval_list_as_function_declaration(list: &Vec<Node>, scope: &Scope) -> RuntimeValue {
    let args = parse_as_args(&list[1]);
    let fn_body = &list[2];

    // todo substitute scope into fn_body
    let _ = scope;

    return RuntimeValue::Function(Function {
        params: args,
        body: fn_body.clone(),
    });
}

fn eval_let(list: &Vec<Node>, scope: &Scope) -> RuntimeValue {
    let binding_exprs = &list[1..list.len() - 1];
    let expr = &list[list.len() - 1]; // ignore both the let and the expr
    let bindings = generate_let_bindings(binding_exprs, scope);
    expr.eval(&scope.with_bindings(bindings))
}

fn eval_if(list: &Vec<Node>, scope: &Scope) -> RuntimeValue {
    let condition = &list[1];
    let if_body = &list[2];
    let else_body = &list[3];
    if condition.eval(scope).bool() {
        if_body.eval(scope)
    } else {
        else_body.eval(scope)
    }
}

fn eval_fun(func: &Function, args: &[RuntimeValue], scope: &Scope) -> RuntimeValue {
    if func.params.len() != args.len() {
        panic!("Function called with incorrect number of arguments");
    }

    // zip the args and params together
    let bindings = func
        .params
        .iter()
        .cloned()
        .zip(args.iter().cloned())
        .collect::<Vec<(String, RuntimeValue)>>();

    func.body.eval(&scope.with_bindings(bindings))
}

fn generate_let_bindings(list: &[Node], scope: &Scope) -> Vec<(String, RuntimeValue)> {
    list.iter()
        .cloned()
        .map(|node| match node {
            Node::List(nodes) => {
                if nodes.len() != 2 {
                    panic!("let binding must be a list of two elements");
                }
                if let Node::Ident(ident) = &nodes[0] {
                    let val = &nodes[1].eval(scope);
                    (ident.clone(), val.clone())
                } else {
                    panic!("left side of let binding must be an identifier");
                }
            }
            _ => panic!("All bindings must be lists"),
        })
        .collect::<Vec<(String, RuntimeValue)>>()
}

fn parse_as_args(expr: &Node) -> Vec<String> {
    if let Node::List(args) = expr {
        args.iter()
            .map(|e| {
                if let Node::Ident(ident) = e {
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
/// and just evaluate it
pub fn interpret(ast: &AST) {
    println!("{:#?}", ast);
    println!("{:#?}", ast.root.eval(&Scope::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let ast = AST {
            root: Node::List(vec![
                Node::Op(Operator::Add),
                Node::Literal(Literal::Numeric(NumericLiteral::Int(1))),
                Node::Literal(Literal::Numeric(NumericLiteral::Int(2))),
            ]),
        };

        let output = ast.root.eval(&Scope::new());

        assert_eq!(output, RuntimeValue::Int(3));
    }

    #[test]
    fn test2() {
        let ast = AST {
            root: Node::List(vec![
                Node::Op(Operator::Add),
                Node::Literal(Literal::Numeric(NumericLiteral::Int(1))),
                Node::Literal(Literal::Numeric(NumericLiteral::Int(2))),
                Node::List(vec![
                    Node::Op(Operator::Sub),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(4))),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(3))),
                ]),
                Node::Literal(Literal::Numeric(NumericLiteral::Int(5))),
                Node::List(vec![
                    Node::Op(Operator::Mul),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(1))),
                    Node::Literal(Literal::Numeric(NumericLiteral::Float(2.3))),
                ]),
            ]),
        };

        let res = ast.root.eval(&Scope::new());
        assert_eq!(res, RuntimeValue::Float(11.3))
    }

    #[test]
    fn test3() {
        let ast = AST {
            root: Node::List(vec![
                Node::Let,
                Node::List(vec![
                    Node::Ident("x".to_owned()),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(2))),
                ]),
                Node::List(vec![
                    Node::Op(Operator::Mul),
                    Node::Ident("x".to_owned()),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(3))),
                ]),
            ]),
        };

        let res = ast.root.eval(&Scope::new());
        assert_eq!(res, RuntimeValue::Int(6))
    }

    #[test]
    fn test_quote() {
        let ast = AST {
            root: Node::List(vec![
                Node::Quote,
                Node::List(vec![
                    Node::Op(Operator::Add),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(1))),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(2))),
                ]),
                Node::Literal(Literal::String("second".to_string()))
            ])
        };

        let res = ast.root.eval(&Scope::new());
        assert_eq!(res, RuntimeValue::List(vec![
            RuntimeValue::Int(3),
            RuntimeValue::String("second".to_string())
        ]));
    }
}
