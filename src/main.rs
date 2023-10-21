mod lexer;
mod parser;

// const INPUT: &str = "(+ 223 1 (* (as) 3.34 2) \"hello\")";
const INPUT: &str = "(\"hello\")";

fn main() {
    let lexed = lexer::lex(INPUT.to_owned());
    println!("{:?}", lexed);
}
