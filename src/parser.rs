use crate::lexer::{Literal, NumericLiteral};
use crate::lexer::{Token, LR};
use crate::sexpr::Sexpr;

#[derive(Debug, PartialEq)]
pub struct Ast {
    pub expressions: Vec<Sexpr>,
}

fn parse_list(rest_tokens: &[Token]) -> Result<(Vec<Sexpr>, usize), String> {
    let mut list = vec![];

    let mut i = 0;
    while i < rest_tokens.len() {
        if rest_tokens[i] == Token::Parenthesis(LR::Right) {
            i += 1;
            break;
        }
        if rest_tokens[i] == Token::Comma {
            let (s_expr, i_diff) = parse_sexpr(&rest_tokens[(i + 1)..])?;
            list.push(Sexpr::CommaUnquote(Box::new(s_expr)));
            i += i_diff + 1;
        } else {
            let (s_expr, i_diff) = parse_sexpr(&rest_tokens[i..])?;
            list.push(s_expr);
            i += i_diff;
        }
    }

    Ok((list, i))
}

pub fn parse_sexpr(rest_tokens: &[Token]) -> Result<(Sexpr, usize), String> {
    let first = &rest_tokens[0];

    match first {
        Token::Parenthesis(LR::Left) => {
            let (sexprs, i_diff) = parse_list(&rest_tokens[1..])?;
            let list = Sexpr::List {
                sexprs,
                quasiquote: false,
            };
            Ok((list, i_diff + 1))
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
            Ok((sexpr, 1))
        }
        Token::Symbol(sym) => Ok((Sexpr::Symbol(sym.clone()), 1)),
        // These should not happen because they are handled in parse_list
        // could this be handled better by tightening up the types?
        // basically it's the responsibility of parse_list to handle these
        // by skipping them and returning the correct index skipper
        Token::Comma | Token::Parenthesis(LR::Right) => {
            Err(format!("Unexpected token: {:?}", first))
        }
        Token::Backtick => {
            let next_token = &rest_tokens[1];
            if let Token::Parenthesis(LR::Left) = next_token {
                let (sexprs, i_diff) = parse_list(&rest_tokens[2..])?;
                let list = Sexpr::List {
                    sexprs,
                    quasiquote: true,
                };
                Ok((list, i_diff + 2))
            } else {
                Err(format!(
                    "Unexpected token after backtick: '{:?}', expected '('",
                    next_token
                ))
            }
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Ast, String> {
    let mut expressions = vec![];
    let mut i = 0;
    let mut rest = &tokens[i..];
    loop {
        let (s_expr, i_diff) = parse_sexpr(rest)?;
        expressions.push(s_expr);
        i += i_diff;
        rest = &tokens[i..];
        if i == tokens.len() {
            break;
        }
        assert!(i < tokens.len()); // for now, expect to parse all tokens from a single s_expr
    }
    let ast = Ast { expressions };
    Ok(ast)
}

#[cfg(test)]
mod tests {
    use crate::lexer::{lex, NumericLiteral};

    use super::*;

    #[test]
    fn test1() -> Result<(), String> {
        let input = vec![Token::Literal(Literal::Numeric(NumericLiteral::Int(123)))];

        assert_eq!(parse(input)?.expressions, vec![Sexpr::Int(123)]);
        Ok(())
    }

    #[test]
    fn test2() -> Result<(), String> {
        let Ast { expressions } = parse(lex(&"(+ 1 (- 4 3))".to_string())?)?;

        assert_eq!(
            expressions,
            vec![Sexpr::List {
                quasiquote: false,
                sexprs: vec![
                    Sexpr::Symbol("+".to_string()),
                    Sexpr::Int(1),
                    Sexpr::List {
                        quasiquote: false,
                        sexprs: vec![Sexpr::Symbol("-".to_string()), Sexpr::Int(4), Sexpr::Int(3)]
                    }
                ]
            }]
        );
        Ok(())
    }

    #[test]
    fn test_quasiquote() -> Result<(), String> {
        let Ast { expressions } = parse(lex(&"(+ 1 `(- 4 3))".to_string())?)?;
        assert_eq!(
            expressions,
            vec![Sexpr::List {
                quasiquote: false,
                sexprs: vec![
                    Sexpr::Symbol("+".to_string()),
                    Sexpr::Int(1),
                    Sexpr::List {
                        quasiquote: true,
                        sexprs: vec![Sexpr::Symbol("-".to_string()), Sexpr::Int(4), Sexpr::Int(3)]
                    }
                ]
            }]
        );
        Ok(())
    }

    #[test]
    fn test_comma_unquote() -> Result<(), String> {
        let Ast { expressions } = parse(lex(&"(+ 1 ,(- 4 3))".to_string())?)?;
        assert_eq!(
            expressions,
            vec![Sexpr::List {
                quasiquote: false,
                sexprs: vec![
                    Sexpr::Symbol("+".to_string()),
                    Sexpr::Int(1),
                    Sexpr::CommaUnquote(Box::new(Sexpr::List {
                        quasiquote: false,
                        sexprs: vec![Sexpr::Symbol("-".to_string()), Sexpr::Int(4), Sexpr::Int(3)]
                    }))
                ]
            }]
        );
        Ok(())
    }

    #[test]
    fn test_comma_unquote_2() -> Result<(), String> {
        let Ast { expressions } = parse(lex(&"(,a ,b)".to_string())?)?;
        assert_eq!(
            expressions,
            vec![Sexpr::List {
                quasiquote: false,
                sexprs: vec![
                    Sexpr::CommaUnquote(Box::new(Sexpr::Symbol("a".to_string()))),
                    Sexpr::CommaUnquote(Box::new(Sexpr::Symbol("b".to_string()))),
                ]
            }]
        );
        Ok(())
    }
}
