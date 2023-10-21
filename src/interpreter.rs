use std::collections::HashMap;

use crate::lexer::{Literal, NumericLiteral, Operator};
use crate::parser::{Node, AST};
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
        for (ident, val) in bindings {
            new_bindings.insert(ident, val);
        }
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
            Node::Boolean(bool) => RuntimeValue::Boolean(*bool),
            Node::Op(op) => RuntimeValue::Op(*op),
            Node::Ident(ident) => {
                match scope.bindings.get(ident) {
                    Some(val) => val.clone(), // inefficient, but for now just clone the value
                    None => panic!("Identifier {} not found in scope", ident),
                }
            }
            Node::Fn | Node::If | Node::Let => panic!("should not be parsed as a node"),
        }
    }
}

fn eval_list(list: &Vec<Node>, scope: &Scope) -> RuntimeValue {
    let first_node = &list[0];

    match first_node {
        // this is a smell - Fn isnt a real SExpr
        Node::Fn => eval_fun_dec(list, scope),
        Node::If => eval_if(list, scope),
        Node::Let => eval_let(list, scope),
        Node::Ident(ident) => {
            let head_val = scope.bindings.get(ident).unwrap();
            let args_vals: &[RuntimeValue] = &list[1..]
                .iter()
                .map(|arg| arg.eval(scope))
                .collect::<Vec<RuntimeValue>>();

            match head_val {
                RuntimeValue::Function(func) => {
                    eval_fun(func, &args_vals, scope)
                }
                _ => panic!("Cannot call non-function value"),
            }
        }
        Node::Op(op) => {
            // for now only handle operators
            let rest_val = &list[1..]
                .iter()
                .map(|e| e.eval(scope))
                .collect::<Vec<RuntimeValue>>();

            op.execute(rest_val)
        }
        Node::Literal(_) | Node::Boolean(_) => panic!(),
        Node::List(_) => todo!(),
    }
}

fn eval_fun_dec(list: &Vec<Node>, scope: &Scope) -> RuntimeValue {
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
    (if condition.eval(scope).bool() {
        if_body
    } else {
        else_body
    }).eval(scope)
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
    println!("AST: {:#?}", &ast);

    return match ast.root.eval(&Scope::new()) {
        RuntimeValue::Boolean(b) => println!("Boolean: {}", b),
        RuntimeValue::Float(f) => println!("Float: {}", f),
        RuntimeValue::Int(i) => println!("Int: {}", i),
        RuntimeValue::String(s) => println!("String: {}", s),
        RuntimeValue::Function(func) => println!("Function: {:#?}", func),
        RuntimeValue::Op(op) => println!("Op: {:#?}", op),
    };
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
}
