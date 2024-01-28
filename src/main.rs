mod builtins;
mod evaluator;
mod lexer;
mod parser;
mod runtime_value;
mod sturctural_parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("Please provide a file path to execute");
    }

    let file_path = &args[1];
    let contents =
        std::fs::read_to_string(file_path).expect("Something went wrong reading the file");

    let tokens = lexer::lex(&contents);
    let ast_root = parser::parse(tokens);
    let structured_ast = sturctural_parser::structure_ast(ast_root);
    evaluator::evaluate(structured_ast);
}

// fn run(input: String) -> String {
//     let tokens = lexer::lex(&input);
//     let ast = parser::parse(tokens);
//     let ast = sturctural_parser::structure_ast(ast);
//     let result = evaluator::evaluate(ast);
//     format!("{:?}", result)
// }
