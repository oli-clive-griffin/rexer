#[derive(Debug, PartialEq)]
pub enum LR {
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Parenthesis(LR),
    Operator(Operator),
    StringLiteral(String),
    IntLiteral(i32),
    FloatLiteral(f32),
    Identifier(String),
    Boolean(bool),
    Comma,
}

impl Symbol {
    fn from_string(s: &String) -> Symbol {
        if s == "true" {
            return Symbol::Boolean(true);
        } else if s == "false" {
            return Symbol::Boolean(false);
        } else {
            return Symbol::Identifier(s.to_string());
        }
    }

    fn from_numeric(s: &String) -> Symbol {
        if s.contains('.') {
            Symbol::FloatLiteral(s.parse::<f32>().expect("Could not parse numeric literal as float"))
        } else {
            Symbol::IntLiteral(s.parse::<i32>().expect("Could not parse numeric literal as int"))
        }
    }

    fn from_char(c: char) -> Symbol {
        match c {
            '(' => Symbol::Parenthesis(LR::Left),
            ')' => Symbol::Parenthesis(LR::Right),
            ',' => Symbol::Comma,
            '+' => Symbol::Operator(Operator::Plus),
            '-' => Symbol::Operator(Operator::Minus),
            '*' => Symbol::Operator(Operator::Multiply),
            '/' => Symbol::Operator(Operator::Divide),
            _ => panic!("Unexpected char: {}", c),
        }
    }
}

enum LexerState {
    None, // single char symbols
    NumberLiteral(String),
    StringLiteral(String), // no escaping, could do by `StringLiteral(Escaped)`
    Identifier(String), // could resolve to a keyword, identifier, or boolean
}

pub fn lex(s: String) -> Vec<Symbol> {
    let mut state: LexerState = LexerState::None;

    let mut symbols: Vec<Symbol> = vec![];

    let chars = s.chars().collect::<Vec<_>>();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        match state {
            LexerState::Identifier(ref mut s) => {
                if c.is_alphanumeric() {
                    s.push(c);
                    i += 1;
                } else {
                    symbols.push(Symbol::from_string(s));
                    state = LexerState::None
                }
            }
            LexerState::NumberLiteral(ref mut s) => {
                if c.is_numeric() || c == '.' {
                    s.push(c);
                    i += 1;
                } else {
                    if c != ' ' && c != '(' && c != ')' && c != ',' {
                        panic!("Unexpected character in number literal: {}", c);
                    }
                    symbols.push(Symbol::from_numeric(&s));
                    state = LexerState::None;
                }
            }
            LexerState::StringLiteral(ref mut s) => {
                if c != '"' {
                    s.push(c);
                    i += 1;
                } else {
                    symbols.push(Symbol::StringLiteral(s.to_string()));
                    state = LexerState::None;
                    i += 1;
                }
            }
            LexerState::None => {
                if ['(', ')', ',', '+', '-', '*', '/'].contains(&c) {
                    symbols.push(Symbol::from_char(c));
                } else if c == '"' {
                    state = LexerState::StringLiteral(String::new());
                } else if c.is_numeric() {
                    state = LexerState::NumberLiteral(c.to_string());
                } else if c.is_alphanumeric() {
                    state = LexerState::Identifier(c.to_string());
                } else if c == ' ' {
                } else {
                    panic!("Unexpected character: {}", c);
                }
                i += 1;
            }
        }
    }
    
    match state {
        LexerState::Identifier(s) => symbols.push(Symbol::from_string(&s)),
        LexerState::NumberLiteral(s) => symbols.push(Symbol::from_numeric(&s)),
        LexerState::StringLiteral(_) => panic!("Unexpected end of input"),
        LexerState::None => (),
    }

    return symbols;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_literal() {
        let input = "123".to_string();
        let expected = vec![Symbol::IntLiteral(123)];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_string_literal() {
        let input = "\"hello\"".to_string();
        let expected = vec![Symbol::StringLiteral("hello".to_string())];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_identifier() {
        let input = "variableName".to_string();
        let expected = vec![Symbol::Identifier("variableName".to_string())];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_operators() {
        let input = "(+ - *)".to_string();
        let expected = vec![
            Symbol::Parenthesis(LR::Left),
            Symbol::Operator(Operator::Plus),
            Symbol::Operator(Operator::Minus),
            Symbol::Operator(Operator::Multiply),
            Symbol::Parenthesis(LR::Right),
        ];
        assert_eq!(lex(input), expected);
    }

    #[test]
    fn test_mixed_input() {
        let input = "(define x 10)".to_string();
        let expected = vec![
            Symbol::Parenthesis(LR::Left),
            Symbol::Identifier("define".to_string()),
            Symbol::Identifier("x".to_string()),
            Symbol::IntLiteral(10),
            Symbol::Parenthesis(LR::Right),
        ];
        assert_eq!(lex(input), expected);
    }

    #[test]
    #[should_panic(expected = "Unexpected character: #")]
    fn test_unexpected_character() {
        let input = "#".to_string();
        lex(input);
    }
}

