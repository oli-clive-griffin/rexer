use rusp::compiler::compile;
use rusp::vm::VM;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if !args.len() == 2 {
        panic!("Expected exactly one argument");
    }

    let contents =
        std::fs::read_to_string(&args[1]).expect("Something went wrong reading the file");

    VM::default().run(compile(&contents))
}
