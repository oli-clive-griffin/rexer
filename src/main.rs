use std::io::Write;

use crate::evaluator::Session;

mod builtins;
mod evaluator;
mod lexer;
mod parser;
mod compiler;
mod vm;
mod obj;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => {
            println!("usage: rusp [filepath]")
        }
    }
}

fn run_file(file_path: &String) {
    let contents =
        std::fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let tokens = lexer::lex(&contents).unwrap_or_else(|e| {
        eprintln!("Lexing error: {}", e);
        std::process::exit(1);
    });

    let ast = parser::parse(tokens).unwrap_or_else(|e| {
        eprintln!("Parsing error: {}", e);
        std::process::exit(1);
    });

    evaluator::evaluate(ast).unwrap_or_else(|e| {
        eprintln!("Evaluation error: {}", e);
        std::process::exit(1);
    });

    std::process::exit(0);
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

        let res = run_string(input, &mut session);

        match res {
            Ok(res) => println!("{}", res),
            Err(e) => println!("\u{001b}[31mError:\u{001b}[0m {}", e),
         }
    }
}

fn run_string(input: String, session: &mut Session) -> Result<parser::Sexpr, String> {
    let tokens = lexer::lex(&input)?;
    let sexpr = parser::parse_sexpr(&tokens)?.0;
    session.eval(sexpr)
}
