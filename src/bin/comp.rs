use rusp::{compiler, lexer, parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if !args.len() == 2 {
        panic!("Expected exactly one argument");
    }
    compile_file( &args[1]);
}

fn compile_file(file_path: &String) {
    let contents = std::fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let tokens = lexer::lex(&contents).unwrap_or_else(|e| {
        eprintln!("Lexing error: {}", e);
        std::process::exit(1);
    });

    let ast = parser::parse(tokens).unwrap_or_else(|e| {
        eprintln!("Parsing error: {}", e);
        std::process::exit(1);
    });

    let bc = compiler::compile_sexprs(ast.expressions);

    rusp::vm::VM::default().run(bc);
}