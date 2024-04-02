#[derive(Debug, PartialEq)]
pub enum LR {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone, Copy)] // todo revisit Clone, Copy
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
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
    Operator(Operator),
    Literal(Literal),
    Symbol(String),
    Comma,
}

impl Token {
    fn from_string(s: &String) -> Token {
        match s.as_str() {
            "true" => Token::Literal(Literal::Boolean(true)),
            "false" => Token::Literal(Literal::Boolean(false)),
            _ => Token::Symbol(s.to_string()),
        }
    }

    fn from_numeric(s: &String) -> Token {
        Token::Literal(Literal::Numeric(if s.contains('.') {
            NumericLiteral::Float(
                s.parse::<f64>()
                    .expect("Could not parse numeric literal as float"),
            )
        } else {
            NumericLiteral::Int(
                s.parse::<i64>()
                    .expect("Could not parse numeric literal as int"),
            )
        }))
    }

    fn from_char(c: char) -> Token {
        match c {
            '(' => Token::Parenthesis(LR::Left),
            ')' => Token::Parenthesis(LR::Right),
            ',' => Token::Comma,
            '+' => Token::Operator(Operator::Add),
            '-' => Token::Operator(Operator::Sub),
            '*' => Token::Operator(Operator::Mul),
            '/' => Token::Operator(Operator::Div),
            _ => panic!("Unexpected char: {}", c),
        }
    }
}

enum LexerState {
    None, // single char tokens
    NumberLiteral(String),
    StringLiteral(String), // no escaping, could do by `StringLiteral(Escaped)`
    Identifier(String),    // could resolve to a keyword, identifier, or boolean
}

fn remove_comments(s: String) -> String {
    s.trim()
        .split('\n')
        .filter(|line| !line.trim().starts_with(';'))
        .collect::<Vec<&str>>()
        .concat()
}

pub fn lex(s: &String) -> Vec<Token> {
    let chars = remove_comments(s.to_string())
        .trim()
        .chars()
        .filter(|c| *c != '\n' && *c != '\r')
        .collect::<Vec<_>>();

    let mut state: LexerState = LexerState::None;
    let mut tokens: Vec<Token> = vec![];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        match state {
            LexerState::Identifier(ref mut s) => {
                if c.is_alphanumeric() {
                    s.push(c);
                    i += 1;
                } else {
                    tokens.push(Token::from_string(s));
                    state = LexerState::None
                }
            }
            LexerState::NumberLiteral(ref mut s) => {
                if c.is_numeric() || c == '.' {
                    s.push(c);
                    i += 1;
                } else {
                    if c != ' ' && c != '(' && c != ')' && c != ',' {
                        panic!("Unexpected character in number literal: `{}`", c);
                    }
                    tokens.push(Token::from_numeric(s));
                    state = LexerState::None;
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
                    '(' | ')' | ',' | '+' | '-' | '*' | '/' => {
                        tokens.push(Token::from_char(c));
                    }
                    '"' => {
                        state = LexerState::StringLiteral(String::new());
                    }
                    c if c.is_numeric() => {
                        state = LexerState::NumberLiteral(c.to_string());
                    }
                    c if c.is_alphanumeric() => {
                        state = LexerState::Identifier(c.to_string());
                    }
                    ' ' => {}
                    _ => {
                        panic!("Unexpected character: `{}`", c);
                    }
                }
                i += 1;
            }
        }
    }

    match state {
        LexerState::Identifier(s) => tokens.push(Token::from_string(&s)),
        LexerState::NumberLiteral(s) => tokens.push(Token::from_numeric(&s)),
        LexerState::StringLiteral(_) => panic!("Unexpected end of input"),
        LexerState::None => (),
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_literal() {
        let input = "123".to_string();
        let expected = vec![Token::Literal(Literal::Numeric(NumericLiteral::Int(123)))];
        assert_eq!(lex(&input), expected);
    }

    #[test]
    fn test_string_literal() {
        let input = "\"hello\"".to_string();
        let expected = vec![Token::Literal(Literal::String("hello".to_string()))];
        assert_eq!(lex(&input), expected);
    }

    #[test]
    fn test_identifier() {
        let input = "variableName".to_string();
        let expected = vec![Token::Symbol("variableName".to_string())];
        assert_eq!(lex(&input), expected);
    }

    #[test]
    fn test_operators() {
        let input = "(+ - *)".to_string();
        let expected = vec![
            Token::Parenthesis(LR::Left),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Sub),
            Token::Operator(Operator::Mul),
            Token::Parenthesis(LR::Right),
        ];
        assert_eq!(lex(&input), expected);
    }

    #[test]
    fn test_mixed_input() {
        let input = "(define x 10)".to_string();
        let expected = vec![
            Token::Parenthesis(LR::Left),
            Token::Symbol("define".to_string()),
            Token::Symbol("x".to_string()),
            Token::Literal(Literal::Numeric(NumericLiteral::Int(10))),
            Token::Parenthesis(LR::Right),
        ];
        assert_eq!(lex(&input), expected);
    }

    #[test]
    #[should_panic(expected = "Unexpected character: `#`")]
    fn test_unexpected_character() {
        let input = "#".to_string();
        lex(&input);
    }
}
