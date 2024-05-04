use std::io::Write;

use crate::{evaluator, lexer, parser, sexpr};

pub fn run_file(file_path: &String) {
    let contents =
        std::fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let tokens = lexer::lex(&contents).unwrap_or_else(|e| {
        panic!("Lexing error: {}", e);
    });

    let ast = parser::parse(tokens).unwrap_or_else(|e| {
        panic!("Parsing error: {}", e);
    });

    evaluator::evaluate(ast).unwrap_or_else(|e| {
        panic!("Evaluation error: {}", e);
    });
}

pub fn repl() {
    println!("\u{001b}[1mRusp\u{001b}[0m");
    let mut session = evaluator::Session::new();

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

fn run_string(input: String, session: &mut evaluator::Session) -> Result<sexpr::LispValue, String> {
    let tokens = lexer::lex(&input)?;
    let sexpr = parser::parse_sexpr(&tokens)?.0;
    session.eval(sexpr.to_sexpr())
}
