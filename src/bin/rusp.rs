use rusp::interpreter::{repl, run_file};

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

