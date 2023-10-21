mod lexer;
mod parser;
mod interpreter;
mod runtime_value;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("Please provide a file path to execute");
    }

    let file_path = &args[1];
    let contents = std::fs::read_to_string(file_path).expect("Something went wrong reading the file").trim().to_owned();

    let tokens = lexer::lex(&contents);
    let ast = parser::parse(tokens);
    println!("{:?}", interpreter::interpret(&ast));
}
