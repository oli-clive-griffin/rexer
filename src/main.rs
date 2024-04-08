use std::io::Write;

use crate::evaluator::Session;

mod builtins;
mod evaluator;
mod lexer;
mod parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => { panic!("asdf") }
    }
}

fn run_file(file_path: &String) {
    let contents =
        std::fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let tokens = lexer::lex(&contents);
    let ast = parser::parse(tokens);
    evaluator::evaluate(ast).unwrap();
}

fn repl() {
    let bold_escape_start = "\u{001b}[1m";
    let bold_escape_end = "\u{001b}[0m";
    println!("{}Rusp{}", bold_escape_start, bold_escape_end);
    let mut session = Session::new();

    loop {
        print!(">> ");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let tokens = lexer::lex(&input);
        let sexpr = parser::parse_sexpr(&tokens).0;
        let res = session.eval(sexpr);
        match res {
            Ok(res) => println!("{}", res),
            Err(e) => println!("\u{001b}[31mError:\u{001b}[0m {}", e),
         }
    }
}
