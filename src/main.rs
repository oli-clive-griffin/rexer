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

enum LexerState {
    None, // single char symbols
    NumberLiteral(String),
    StringLiteral(String), // no escaping, could do by `StringLiteral(Escaped)`
    Identifier(String),
}

fn lex(s: String) -> Vec<Symbol> {
    let mut state: LexerState = LexerState::None;

    let mut symbols: Vec<Symbol> = vec![];

    let mut i = 0;
    while let Some(c) = s.chars().nth(i) {
        match state {
            LexerState::Identifier(ref mut s) => {
                if c.is_alphanumeric() {
                    s.push(c);
                    i += 1;
                } else {
                    symbols.push(Symbol::Identifier(s.to_string()));
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

    return symbols;
}


const INPUT: &str = "(+ 223 1 (* (as) 3.34 2) \"hello\")";

fn main() {
    let lexed = lex(INPUT.to_owned());
    println!("{:?}", lexed);
}

