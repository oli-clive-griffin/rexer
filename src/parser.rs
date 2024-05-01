use crate::lexer::{Literal, NumericLiteral};
use crate::lexer::{Token, LR};
use crate::sexpr::SrcSexpr;

#[derive(Debug, PartialEq)]
pub struct Ast {
    pub expressions: Vec<SrcSexpr>,
}

fn parse_list(rest_tokens: &[Token]) -> Result<(Vec<SrcSexpr>, usize), String> {
    let mut list = vec![];

    let mut i = 0;
    while i < rest_tokens.len() {
        if rest_tokens[i] == Token::Parenthesis(LR::Right) {
            i += 1;
            break;
        }
        if rest_tokens[i] == Token::Comma {
            panic!();
            // let (s_expr, i_diff) = parse_sexpr(&rest_tokens[(i + 1)..])?;
            // list.push(SrcSexpr::CommaUnquote(Box::new(s_expr)));
            // i += i_diff + 1;
        } else {
            let (s_expr, i_diff) = parse_sexpr(&rest_tokens[i..])?;
            list.push(s_expr);
            i += i_diff;
        }
    }

    Ok((list, i))
}

pub fn parse_sexpr(rest_tokens: &[Token]) -> Result<(SrcSexpr, usize), String> {
    let first = &rest_tokens[0];

    match first {
        Token::Parenthesis(LR::Left) => {
            let (sexprs, i_diff) = parse_list(&rest_tokens[1..])?;
            let list = SrcSexpr::List(sexprs);
            Ok((list, i_diff + 1))
        }
        Token::Literal(lit) => {
            let sexpr = match lit {
                Literal::Numeric(num) => match num {
                    NumericLiteral::Int(i) => SrcSexpr::Int(*i),
                    NumericLiteral::Float(f) => SrcSexpr::Float(*f),
                },
                Literal::String(s) => SrcSexpr::String(s.clone()),
                Literal::Boolean(b) => SrcSexpr::Bool(*b),
            };
            Ok((sexpr, 1))
        }
        Token::Symbol(sym) => Ok((SrcSexpr::Symbol(sym.clone()), 1)),
        // These should not happen because they are handled in parse_list
        // could this be handled better by tightening up the types?
        // basically it's the responsibility of parse_list to handle these
        // by skipping them and returning the correct index skipper
        Token::Comma | Token::Parenthesis(LR::Right) => {
            Err(format!("Unexpected token: {:?}", first))
        }
        Token::Backtick => {
            panic!()
        }
        Token::Apostrophe => {
            parse_sexpr(&rest_tokens[1..]).map(|op| {
                (
                    SrcSexpr::Quote(Box::new(op.0)), //
                    op.1 + 1,                        // skip the apostrophe
                )
            })
        } // match next_token {
          //     Token::Parenthesis(LR::Left) => {
          //         let (sexprs, i_diff) = parse_list(&rest_tokens[2..])?;
          //         let list = SrcSexpr::Quote(Box::new(SrcSexpr::List(sexprs)));
          //         Ok((list, i_diff + 2))
          //     }
          //     Token::Symbol(sym) => {
          //         let sexpr = SrcSexpr::Quote(Box::new(SrcSexpr::Symbol(sym.clone())));
          //         Ok((sexpr, 2))
          //     }
          //     Token::Literal(lit) => {
          //         let sexpr = match lit {
          //             Literal::Numeric(num) => match num {
          //                 NumericLiteral::Int(i) => SrcSexpr::Quote(Box::new(SrcSexpr::Integer(*i))),
          //                 NumericLiteral::Float(f) => SrcSexpr::Quote(Box::new(SrcSexpr::Float(*f))),
          //             },
          //             Literal::String(s) => SrcSexpr::Quote(Box::new(SrcSexpr::String(s.clone()))),
          //             Literal::Boolean(b) => SrcSexpr::Quote(Box::new(SrcSexpr::Bool(*b))),
          //         };
          //         Ok((sexpr, 2))
          //     }
          //     Token::Comma => todo!(),
          //     Token::Apostrophe => todo!(),
          //     Token::Backtick => todo!(),

          // }
          // if let Token::Parenthesis(LR::Left) = next_token {
          //     let (sexprs, i_diff) = parse_list(&rest_tokens[2..])?;
          //     let list =  SrcSexpr::Quote(Box::new(SrcSexpr::List(sexprs)));
          //     Ok((list, i_diff + 2))
          // } else {
          //     Err(format!(
          //         "Unexpected token after backtick: '{:?}', expected '('",
          //         next_token
          //     ))
          // }

          // Token::Backtick | Token::Apostrophe => {
          // let next_token = &rest_tokens[1];
          // if let Token::Parenthesis(LR::Left) = next_token {
          //     let (sexprs, i_diff) = parse_list(&rest_tokens[2..])?;
          //     let list = match first {
          //         Token::Backtick => SrcSexpr::DEP_QuasiQuotedList(sexprs),
          //         Token::Apostrophe => SrcSexpr::Quote(Box::new(SrcSexpr::List(sexprs))),
          //         _ => unreachable!(),
          //     };
          //     Ok((list, i_diff + 2))
          // } else {
          //     Err(format!(
          //         "Unexpected token after backtick: '{:?}', expected '('",
          //         next_token
          //     ))
          // }
          // }
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
        if i >= tokens.len() {
            panic!();
        } // for now, expect to parse all tokens from a single s_expr
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

        assert_eq!(parse(input)?.expressions, vec![SrcSexpr::Int(123)]);
        Ok(())
    }

    #[test]
    fn test2() -> Result<(), String> {
        let Ast { expressions } = parse(lex(&"(+ 1 (- 4 3))".to_string())?)?;

        assert_eq!(
            expressions,
            vec![SrcSexpr::List(vec![
                SrcSexpr::Symbol("+".to_string()),
                SrcSexpr::Int(1),
                SrcSexpr::List(vec![
                    SrcSexpr::Symbol("-".to_string()),
                    SrcSexpr::Int(4),
                    SrcSexpr::Int(3)
                ])
            ])]
        );
        Ok(())
    }

    // #[test]
    // fn test_quote_level() -> Result<(), String> {
    //     let Ast { expressions } = parse(lex(&"(+ 1 `(- 4 3))".to_string())?)?;
    //     assert_eq!(
    //         expressions,
    //         vec![SrcSexpr::List(vec![
    //             SrcSexpr::Symbol("+".to_string()),
    //             SrcSexpr::Int(1),
    //             SrcSexpr::DEP_QuasiQuotedList(vec![
    //                 SrcSexpr::Symbol("-".to_string()),
    //                 SrcSexpr::Int(4),
    //                 SrcSexpr::Int(3)
    //             ])
    //         ])]
    //     );
    //     Ok(())
    // }

    // #[test]
    // fn test_comma_unquote() -> Result<(), String> {
    //     let Ast { expressions } = parse(lex(&"(+ 1 ,(- 4 3))".to_string())?)?;
    //     assert_eq!(
    //         expressions,
    //         vec![SrcSexpr::List(vec![
    //             SrcSexpr::Symbol("+".to_string()),
    //             SrcSexpr::Int(1),
    //             SrcSexpr::CommaUnquote(Box::new(SrcSexpr::List(vec![
    //                 SrcSexpr::Symbol("-".to_string()),
    //                 SrcSexpr::Int(4),
    //                 SrcSexpr::Int(3)
    //             ])))
    //         ])]
    //     );
    //     Ok(())
    // }

    // #[test]
    // fn test_comma_unquote_2() -> Result<(), String> {
    //     let Ast { expressions } = parse(lex(&"(,a ,b)".to_string())?)?;
    //     assert_eq!(
    //         expressions,
    //         vec![SrcSexpr::List(vec![
    //             SrcSexpr::CommaUnquote(Box::new(SrcSexpr::Symbol("a".to_string()))),
    //             SrcSexpr::CommaUnquote(Box::new(SrcSexpr::Symbol("b".to_string()))),
    //         ])]
    //     );
    //     Ok(())
    // }
}
