#[derive(Debug, PartialEq)]
enum LR {
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
enum Symbol {
    Parenthesis(LR),
    Operator(Operator),
    StringLiteral(String),
    IntLiteral(i32),
    FloatLiteral(f32),
    Identifier(String),
    Comma,
}

impl Symbol {
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

const INPUT: &str = "(+ 223 1 (* (as) 3.34 2) \"hello\")";

enum LexerState {
    None, // single char symbols
    NumberLiteral(String),
    StringLiteral(String), // no escaping, could do by `StringLiteral(Escaped)`
    Identifier(String),
}

fn lex(s: &str) -> Vec<Symbol> {
    if !s.starts_with('(') {
        panic!("Input must start with '('")
    }

    let mut state: LexerState = LexerState::None;

    let mut symbols: Vec<Symbol> = vec![];

    for c in s.chars() {
        match state {
            LexerState::Identifier(ref mut s) => {
                if c.is_alphanumeric() {
                    s.push(c);
                } else {
                    symbols.push(Symbol::Identifier(s.to_string()));

                    if ['(', ')', ','].contains(&c) {
                        symbols.push(Symbol::from_char(c));
                    }

                    state = LexerState::None
                }
            }
            LexerState::NumberLiteral(ref mut s) => {
                if c.is_numeric() || c == '.' {
                    s.push(c);
                } else {
                    if c != ' ' && c != '(' && c != ')' && c != ',' {
                        panic!("Unexpected character in number literal: {}", c);
                    }
                    symbols.push(Symbol::from_numeric(&s));

                    if ['(', ')', ','].contains(&c) {
                        symbols.push(Symbol::from_char(c));
                    }

                    state = LexerState::None;
                }
            }
            LexerState::StringLiteral(ref mut s) => {
                if c == '"' {
                    symbols.push(Symbol::StringLiteral(s.to_string()));
                    state = LexerState::None;
                } else {
                    s.push(c);
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
            }
        }
    }

    return symbols;
}

fn main() {
    let lexed = lex(INPUT);
    println!("{:?}", lexed);
}

