use core::panic;
use std::collections::HashMap;

use crate::builtins::BUILTINTS;
use crate::function::Function;
use crate::lexer::{Literal, NumericLiteral, Operator};
use crate::parser::Node;
use crate::runtime_value::RuntimeValue;
use crate::sturctural_parser::{Form, SpecialForm, StructuredNode};

struct Scope {
    // could make this a list of hashmaps that's search from the top down
    // would negate the need to duplicate the scope when adding items
    bindings: HashMap<String, RuntimeValue>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            bindings: HashMap::from_iter(
                BUILTINTS.map(|(name, builtin)| (name.to_owned(), RuntimeValue::BuiltIn(builtin))),
            ),
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

impl Function {
    fn eval(self, args: &[RuntimeValue], scope: &Scope) -> RuntimeValue {
        if self.params.len() != args.len() {
            panic!("Function called with incorrect number of arguments");
        }

        // zip the args and params together
        let bindings = self
            .params
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect::<Vec<(String, RuntimeValue)>>();

        self.body.eval(&scope.with_bindings(bindings))
    }
}

impl RuntimeValue {
    fn from_literal(lit: Literal) -> RuntimeValue {
        match lit {
            Literal::Numeric(n) => match n {
                NumericLiteral::Int(i) => RuntimeValue::Int(i),
                NumericLiteral::Float(f) => RuntimeValue::Float(f),
            },
            Literal::String(s) => RuntimeValue::String(s),
            Literal::Boolean(b) => RuntimeValue::Boolean(b),
        }
    }
}

impl Operator {
    fn eval(&self, args: &[RuntimeValue]) -> RuntimeValue {
        args.iter()
            .cloned()
            .reduce(|acc, val| self.binary(acc, val))
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

impl StructuredNode {
    fn eval(self, scope: &Scope) -> RuntimeValue {
        match self {
            StructuredNode::Form(form) => form.eval(scope),
            StructuredNode::Ident(ident) => scope
                .bindings
                .get(&ident)
                .expect(format!("Identifier {ident} not found in scope").as_str())
                .clone(),
            StructuredNode::Literal(lit) => RuntimeValue::from_literal(lit),
            StructuredNode::Op(op) => RuntimeValue::Op(op),
        }
    }
}

impl SpecialForm {
    fn eval(self, scope: &Scope) -> RuntimeValue {
        match self {
            SpecialForm::Fn(form) => {
                // form.args
                // let args = parse_as_args(&form.args);
                // let fn_body = &form.body;

                // todo substitute scope into fn_body
                // let _ = scope;

                RuntimeValue::Function(Function {
                    params: form.args,
                    body: *form.body,
                })
            }
            SpecialForm::If(form) => {
                if form.condition.eval(scope).bool() {
                    form.if_body.eval(scope)
                } else {
                    form.else_body.eval(scope)
                }
            }
            SpecialForm::Let(form) => {
                let bindings = form_let_bindings(&form.bindings, scope);
                form.expr.eval(&scope.with_bindings(bindings))
            }
            SpecialForm::Quote(form) => quote(form.expr),
        }
    }
}

impl Form {
    fn eval(self, scope: &Scope) -> RuntimeValue {
        match self {
            Form::Special(form) => form.eval(scope),
            Form::Regular(node) => eval_normal_form(node, scope),
        }
    }
}

fn eval_normal_form(list: Vec<StructuredNode>, scope: &Scope) -> RuntimeValue {
    let vals = list
        .iter()
        .map(|arg| arg.clone().eval(scope))
        .collect::<Vec<RuntimeValue>>();

    let head_val = vals[0].clone();
    let args_vals = &vals[1..];

    match head_val {
        RuntimeValue::Op(op) => op.eval(args_vals),
        RuntimeValue::BuiltIn(builtin) => builtin.eval(args_vals),
        RuntimeValue::Function(func) => func.clone().eval(args_vals, scope),
        RuntimeValue::Int(_) => panic!("Cannot call int value. list: {:?}", list),
        RuntimeValue::List(_) => panic!("Cannot call list value"),
        RuntimeValue::Float(_) => panic!("Cannot call float value"),
        RuntimeValue::String(_) => panic!("Cannot call string value"),
        RuntimeValue::Boolean(_) => panic!("Cannot call boolean value"),
        RuntimeValue::Symbol(_) => panic!("Cannot call symbol value"),
    }
}

fn quote(node: Node) -> RuntimeValue {
    match node {
        Node::Ident(ident) => RuntimeValue::Symbol(ident.clone()),
        Node::Literal(lit) => RuntimeValue::from_literal(lit),
        Node::Op(op) => RuntimeValue::Op(op),
        Node::List(node_list) => {
            RuntimeValue::List(node_list.iter().cloned().map(|node| quote(node)).collect())
        }
        Node::Fn => todo!("not sure how we should handle quoting `Node::Fn`"),
        Node::If => todo!("not sure how we should handle quoting `Node::If`"),
        Node::Let => todo!("not sure how we should handle quoting `Node::Let`"),
        Node::Quote => todo!("not sure how we should handle quoting `Node::Quote`"),
    }
}

fn form_let_bindings(
    list: &[(String, StructuredNode)],
    scope: &Scope,
) -> Vec<(String, RuntimeValue)> {
    list.iter()
        .cloned()
        .map(|(name, expr)| (name, expr.eval(scope)))
        .collect::<Vec<(String, RuntimeValue)>>()
}

/// for now, assume that the AST is a single SExpr
/// and just evaluate it.
/// Obvious next steps are to allow for multiple SExprs (lines)
/// and to manage a global scope being passed between them.
pub fn evaluate(ast: StructuredNode) {
    println!("{:#?}", ast.eval(&Scope::new()));
}

#[cfg(test)]
mod tests {
    use crate::sturctural_parser::structure_ast;

    use super::*;

    #[test]
    fn test1() {
        let ast = structure_ast(Node::List(vec![
            Node::Op(Operator::Add),
            Node::Literal(Literal::Numeric(NumericLiteral::Int(1))),
            Node::Literal(Literal::Numeric(NumericLiteral::Int(2))),
        ]));
        let output = ast.eval(&Scope::new());
        assert_eq!(output, RuntimeValue::Int(3));
    }

    #[test]
    fn test2() {
        let ast = structure_ast(Node::List(vec![
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
        ]));
        let res = ast.eval(&Scope::new());
        assert_eq!(res, RuntimeValue::Float(11.3))
    }

    #[test]
    fn test3() {
        let ast = structure_ast(Node::List(vec![
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
        ]));
        let res = ast.eval(&Scope::new());
        assert_eq!(res, RuntimeValue::Int(6))
    }
}
