use std::io::Write;

use rusp::compiler::compile;
use rusp::vm::VM;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match &args[..] {
        [_, file] => interpret(file),
        [_] => repl(),
        _ => panic!("Usage: ruspc [filename]"),
    }
}

fn interpret(filename: &str) {
    let contents =
        std::fs::read_to_string(filename).expect("Something went wrong reading the file");

    VM::default().run(compile(&contents))
}

fn repl() {
    let mut vm = VM::default();

    println!("RUSP");
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();
        // let input = format!("(print {})", &input);
        vm.run(compile(&input));
    }
}
