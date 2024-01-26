pub use crate::lexer::{Literal, NumericLiteral, Operator};
use crate::lexer::{Token, LR};

#[derive(Debug, PartialEq)]
pub struct AST {
    pub root: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    List(Vec<Node>), // doesn't allow for quoting, but this does: // List(Quoted, Vec<SExpr>), // impl later
    Ident(String),
    Literal(Literal),
    Op(Operator),
    Fn,
    If,
    Let,
    Quote,
    // impl later:
    // Def,
}

fn parse_list(rest_tokens: &[Token]) -> (Node, usize) {
    let mut list = vec![];

    let mut i = 0;
    while i < rest_tokens.len() {
        if rest_tokens[i] == Token::Parenthesis(LR::Right) {
            i += 1;
            break;
        }
        if rest_tokens[i] == Token::Comma {
            i += 1;
            continue;
        }
        let (s_expr, i_diff) = parse_sexpr(&rest_tokens[i..]);
        list.push(s_expr);
        i += i_diff;
    }

    (Node::List(list), i)
}

fn parse_sexpr(rest_tokens: &[Token]) -> (Node, usize) {
    let first = &rest_tokens[0];

    match first {
        Token::Parenthesis(LR::Left) => {
            let (s_expr, i_diff) = parse_list(&rest_tokens[1..]);
            (s_expr, i_diff + 1)
        }
        Token::Operator(op) => (Node::Op(*op), 1), // copied
        Token::Literal(lit) => (Node::Literal(lit.clone()), 1),
        Token::Identifier(ident) => match ident.as_str() {
            "fn" => (Node::Fn, 1),
            "if" => (Node::If, 1),
            "let" => (Node::Let, 1),
            "quote" => (Node::Quote, 1),
            _ => (Node::Ident(ident.clone()), 1),
        },
        // These should not happen because they are handled in parse_list
        // could this be handled better by tightening up the types?
        // basically it's the responsibility of parse_list to handle these
        // by skipping them and returning the correct index skipper
        Token::Comma | Token::Parenthesis(LR::Right) => {
            panic!("Unexpected token: {:?}", first);
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> AST {
    let (s_expr, i_diff) = parse_sexpr(&tokens[..]);
    assert!(i_diff == tokens.len()); // for now, expect to parse all tokens from a single s_expr
    AST { root: s_expr }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{lex, NumericLiteral};

    use super::*;

    #[test]
    fn test1() {
        let input = vec![Token::Literal(Literal::Numeric(NumericLiteral::Int(123)))];

        assert_eq!(
            parse(input).root,
            Node::Literal(Literal::Numeric(NumericLiteral::Int(123)))
        );
    }

    #[test]
    fn test2() {
        let AST { root } = parse(lex(&"(+ 1 (- 4 3))".to_string()));

        assert_eq!(
            root,
            Node::List(vec![
                Node::Op(Operator::Add),
                Node::Literal(Literal::Numeric(NumericLiteral::Int(1))),
                Node::List(vec![
                    Node::Op(Operator::Sub),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(4))),
                    Node::Literal(Literal::Numeric(NumericLiteral::Int(3))),
                ])
            ])
        );
    }
}
