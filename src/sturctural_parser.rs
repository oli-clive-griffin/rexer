use crate::{
    lexer::{Literal, Operator},
    parser::Node,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Form {
    Special(SpecialForm),
    Regular(Vec<StructuredNode>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnForm {
    pub args: Vec<String>,
    pub body: Box<StructuredNode>, // for example, we might want: (let (get-size (fn () 1)) (get-size))
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfForm {
    pub condition: Box<StructuredNode>,
    pub if_body: Box<StructuredNode>,
    pub else_body: Box<StructuredNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LetForm {
    pub bindings: Vec<(String, StructuredNode)>,
    pub expr: Box<StructuredNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct QuoteForm {
    pub expr: Node, // I think this can be anything
}

#[derive(Debug, PartialEq, Clone)]
pub enum SpecialForm {
    Fn(FnForm),
    If(IfForm),
    Let(LetForm),
    Quote(QuoteForm),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructuredNode {
    Form(Form),
    Ident(String),
    Literal(Literal),
    Op(Operator), // TODO just use builtin
}

pub fn structure_ast(root: Node) -> StructuredNode {
    match root {
        Node::List(nodes) => StructuredNode::Form(parse_form(nodes)), // .iter().map(|node| structure_ast(node)).flatten().collect(),
        Node::Ident(ident) => StructuredNode::Ident(ident.clone()),
        Node::Literal(lit) => StructuredNode::Literal(lit.clone()),
        Node::Op(op) => StructuredNode::Op(op.clone()),
        Node::Fn => panic!("Node::Fn should not be evaluated"),
        Node::If => panic!("Node::If should not be evaluated"),
        Node::Let => panic!("Node::Let should not be evaluated"),
        Node::Quote => panic!("Node::Quote should not be evaluated"),
    }
}

/// structurally parses a list into a form so that structural checking can
/// be decoupled from evaluation
fn parse_form(list: Vec<Node>) -> Form {
    match list[0] {
        Node::Fn => match &list[1..] {
            [Node::List(args), bodyexpr] => {
                let args = args
                    .iter()
                    .map(|arg_node| match arg_node {
                        Node::Ident(ident) => ident.clone(),
                        _ => panic!("Function arguments must be identifiers"),
                    })
                    .collect();
                Form::Special(SpecialForm::Fn(FnForm {
                    args,
                    body: Box::new(structure_ast(bodyexpr.clone())),
                }))
            }
            _ => panic!("Fn form must be called with a list of arguments and a body"),
        },
        Node::If => match &list[1..] {
            [condition, if_body, else_body] => Form::Special(SpecialForm::If(IfForm {
                condition: Box::new(structure_ast(condition.clone())),
                if_body: Box::new(structure_ast(if_body.clone())),
                else_body: Box::new(structure_ast(else_body.clone())),
            })),
            _ => panic!("If form must be called with a condition, if body, and else body"),
        },
        Node::Let => {
            if list.len() < 3 {
                panic!("Let form must be called with a list of bindings and an expr");
            }
            let bindings = &list[1..list.len() - 1];
            let expr = list.last().unwrap();
            return Form::Special(SpecialForm::Let(LetForm {
                bindings: bindings
                    .iter()
                    .map(|node| match node {
                        Node::List(nodes) => match &nodes[..] {
                            [Node::Ident(ident), expr] => {
                                (ident.clone(), structure_ast(expr.clone()))
                            }
                            _ => panic!("let binding must be a list of two elements"),
                        },
                        _ => panic!("All bindings must be lists"),
                    })
                    .collect(),
                expr: Box::new(structure_ast(expr.clone())),
            }));
        }
        Node::Quote => match &list[1..] {
            [expr] => Form::Special(SpecialForm::Quote(QuoteForm { expr: expr.clone() })),
            _ => panic!("Quote form must be called with an expr"),
        },
        _ => Form::Regular(
            list.iter()
                .map(|node| structure_ast(node.clone()))
                .collect(),
        ),
    }
}
