#[derive(Debug, PartialEq)]
pub enum LR {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NumericLiteral {
    Float(f64),
    Int(i64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Numeric(NumericLiteral),
    String(String),
    Boolean(bool),
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Parenthesis(LR),
    Literal(Literal),
    Symbol(String),
    Comma,
    Backtick,   // ` for quote_level
    Apostrophe, // ' for quote
}

impl Token {
    fn from_string(s: &str) -> Token {
        match s {
            "true" => Token::Literal(Literal::Boolean(true)),
            "false" => Token::Literal(Literal::Boolean(false)),
            _ => Token::Symbol(s.to_string()),
        }
    }

    fn from_numeric(s: &str) -> Result<Token, String> {
        Ok(Token::Literal(Literal::Numeric(if s.contains('.') {
            NumericLiteral::Float(s.parse::<f64>().map_err(|e| e.to_string())?)
        } else {
            NumericLiteral::Int(s.parse::<i64>().map_err(|e| e.to_string())?)
        })))
    }
}

enum LexerState {
    None, // single char tokens
    NumberLiteral(String),
    StringLiteral(String), // no escaping, could do by `StringLiteral(Escaped)`
    Symbol(String),        // could resolve to a keyword, identifier, or boolean
}

pub fn lex(s: &String) -> Result<Vec<Token>, String> {
    let chars = s.to_string().trim().chars().collect::<Vec<_>>();

    let mut state: LexerState = LexerState::None;
    let mut tokens: Vec<Token> = vec![];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        match state {
            LexerState::Symbol(ref mut s) => {
                match c {
                    ' ' | '(' | ')' | '\n' => {
                        // potential newline troubles with encoding?
                        tokens.push(Token::from_string(s));
                        state = LexerState::None;
                    }
                    ';' => {
                        // comment, skip to end of line
                        while i < chars.len() && chars[i] != '\n' {
                            i += 1;
                        }
                    }
                    // 'a'..='z' | 'A'..='Z' | '_' | '#' | '-' | ':' => {
                    c => {
                        s.push(c);
                        i += 1;
                    } // _ => {
                      //     return Err(format!("Unexpected character in symbol: [{}]", c).to_string());
                      // }
                }
            }
            LexerState::NumberLiteral(ref mut s) => {
                if c.is_numeric() || c == '.' {
                    s.push(c);
                    i += 1;
                } else if c == ' ' || c == '(' || c == ')' || c == ',' || c == '`' {
                    tokens.push(Token::from_numeric(s)?);
                    state = LexerState::None;
                    // important to not increment i here, we want to lex the current char
                } else {
                    return Err(
                        format!("Unexpected character in number literal: [{}]", c).to_string()
                    );
                }
            }
            LexerState::StringLiteral(ref mut s) => {
                if c != '"' {
                    s.push(c);
                    i += 1;
                } else {
                    tokens.push(Token::Literal(Literal::String(s.to_string())));
                    state = LexerState::None;
                    i += 1;
                }
            }
            LexerState::None => {
                match c {
                    ' ' | '\n' => {}
                    '"' => {
                        state = LexerState::StringLiteral(String::new());
                    }
                    c if c.is_numeric() => {
                        state = LexerState::NumberLiteral(c.to_string());
                    }
                    '(' => tokens.push(Token::Parenthesis(LR::Left)),
                    ')' => tokens.push(Token::Parenthesis(LR::Right)),
                    ',' => tokens.push(Token::Comma),
                    '`' => tokens.push(Token::Backtick),
                    '\'' => tokens.push(Token::Apostrophe),
                    ';' => {
                        // comment, skip to end of line
                        while i < chars.len() && chars[i] != '\n' {
                            i += 1;
                        }
                    }
                    // 'a'..='z' | 'A'..='Z' | '_' | '#' | '-' | ':' => {
                    c => {
                        state = LexerState::Symbol(c.to_string());
                    } // c => panic!("Unexpected character: [{}]", c),
                }
                i += 1;
            }
        }
    }

    match state {
        LexerState::Symbol(s) => tokens.push(Token::from_string(&s)),
        LexerState::NumberLiteral(s) => tokens.push(Token::from_numeric(&s)?),
        LexerState::StringLiteral(_) => return Err("Unexpected end of input".to_string()),
        LexerState::None => (),
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_literal() -> Result<(), String> {
        let input = "123".to_string();
        let expected = vec![Token::Literal(Literal::Numeric(NumericLiteral::Int(123)))];
        assert_eq!(lex(&input)?, expected);
        Ok(())
    }

    #[test]
    fn test_string_literal() -> Result<(), String> {
        let input = "\"hello\"".to_string();
        let expected = vec![Token::Literal(Literal::String("hello".to_string()))];
        assert_eq!(lex(&input)?, expected);
        Ok(())
    }

    #[test]
    fn test_identifier() -> Result<(), String> {
        let input = "variableName".to_string();
        let expected = vec![Token::Symbol("variableName".to_string())];
        assert_eq!(lex(&input)?, expected);
        Ok(())
    }

    #[test]
    fn test_mixed_input() -> Result<(), String> {
        let input = "(define x 10)".to_string();
        let expected = vec![
            Token::Parenthesis(LR::Left),
            Token::Symbol("define".to_string()),
            Token::Symbol("x".to_string()),
            Token::Literal(Literal::Numeric(NumericLiteral::Int(10))),
            Token::Parenthesis(LR::Right),
        ];
        assert_eq!(lex(&input)?, expected);
        Ok(())
    }

    #[test]
    fn test_unexpected_character() -> Result<(), String> {
        let input = "#".to_string();
        let expected = vec![Token::Symbol("#".to_string())];
        assert_eq!(lex(&input)?, expected);
        Ok(())
    }

    #[test]
    fn test_unquote() -> Result<(), String> {
        let input = "(,a ,b)".to_string();
        let expected = vec![
            Token::Parenthesis(LR::Left),
            Token::Comma,
            Token::Symbol("a".to_string()),
            Token::Comma,
            Token::Symbol("b".to_string()),
            Token::Parenthesis(LR::Right),
        ];
        assert_eq!(lex(&input)?, expected);
        Ok(())
    }

    #[test]
    fn test_comment() -> Result<(), String> {
        let input = r#"
(; comment
    a b)
"#
        .to_string();
        let expected = vec![
            Token::Parenthesis(LR::Left),
            Token::Symbol("a".to_string()),
            Token::Symbol("b".to_string()),
            Token::Parenthesis(LR::Right),
        ];
        assert_eq!(lex(&input)?, expected);
        Ok(())
    }

    #[test]
    fn test_newlines() -> Result<(), String> {
        let input = r#"
(a
 b
)
"#
        .to_string();
        let expected = vec![
            Token::Parenthesis(LR::Left),
            Token::Symbol("a".to_string()),
            Token::Symbol("b".to_string()),
            Token::Parenthesis(LR::Right),
        ];
        assert_eq!(lex(&input)?, expected);
        Ok(())
    }
}
