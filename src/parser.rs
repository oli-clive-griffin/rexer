pub use crate::lexer::{Literal, NumericLiteral};
use crate::{builtins::BuiltIn, lexer::{Token, LR}};

#[derive(Debug, PartialEq)]
pub struct Ast {
    pub root: Sexpr,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Sexpr {
    List(Vec<Sexpr>), // doesn't allow for quoting, but this does: // List(Quoted, Vec<SExpr>), // impl later
    Symbol(String),
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Lambda {
        parameters: Vec<String>,
        body: Box<Sexpr>,
    },
    Macro {
        parameters: Vec<String>,
        body: Box<Sexpr>,
    },
    BuiltIn(BuiltIn)
}

fn parse_list(rest_tokens: &[Token]) -> (Sexpr, usize) {
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

    (Sexpr::List(list), i)
}

fn parse_sexpr(rest_tokens: &[Token]) -> (Sexpr, usize) {
    let first = &rest_tokens[0];

    match first {
        Token::Parenthesis(LR::Left) => {
            let (s_expr, i_diff) = parse_list(&rest_tokens[1..]);
            (s_expr, i_diff + 1)
        }
        Token::Literal(lit) => {
            let sexpr = match lit {
                Literal::Numeric(num) => match num {
                    NumericLiteral::Int(i) => Sexpr::Int(*i),
                    NumericLiteral::Float(f) => Sexpr::Float(*f),
                },
                Literal::String(s) => Sexpr::String(s.clone()),
                Literal::Boolean(b) => Sexpr::Bool(*b),
            };
            (sexpr, 1)
        }
        Token::Symbol(sym) => (Sexpr::Symbol(sym.clone()), 1),
        // These should not happen because they are handled in parse_list
        // could this be handled better by tightening up the types?
        // basically it's the responsibility of parse_list to handle these
        // by skipping them and returning the correct index skipper
        Token::Comma | Token::Parenthesis(LR::Right) => {
            panic!("Unexpected token: {:?}", first);
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Ast {
    let (s_expr, i_diff) = parse_sexpr(&tokens[..]);
    assert!(i_diff == tokens.len()); // for now, expect to parse all tokens from a single s_expr
    Ast { root: s_expr }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{lex, NumericLiteral};

    use super::*;

    #[test]
    fn test1() {
        let input = vec![Token::Literal(Literal::Numeric(NumericLiteral::Int(123)))];

        assert_eq!(parse(input).root, Sexpr::Int(123));
    }

    #[test]
    fn test2() {
        let Ast { root } = parse(lex(&"(+ 1 (- 4 3))".to_string()));

        assert_eq!(
            root,
            Sexpr::List(vec![
                Sexpr::Symbol("+".to_string()),
                Sexpr::Int(1),
                Sexpr::List(vec![
                    Sexpr::Symbol("-".to_string()),
                    Sexpr::Int(4),
                    Sexpr::Int(3)
                ])
            ])
        );
    }
}
