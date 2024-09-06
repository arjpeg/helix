use std::{
    env, fs,
    io::{self, Write},
};

use owo_colors::OwoColorize;

use helix::program::Program;

fn main() {
    match env::args().nth(1) {
        Some(path) => run_file(&path),
        None => repl(),
    }
}

fn run_file(path: &str) {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!(
                "{}: {}",
                "Error".red().bold(),
                format!("file path {path:?} does not exist").bold()
            );

            return;
        }
    };

    let mut program = Program::new();
    let main = program.add_source(path.to_owned(), content);

    match program.run(main) {
        Ok(_) => {}
        Err(e) => program.pretty_print_error(e),
    }
}

fn repl() {
    let mut program = Program::new();

    loop {
        let mut line = String::new();
        print!("{} > ", "helix".green());

        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line).unwrap();

        let main = program.add_source("<stdin>".to_string(), line);

        match program.run(main) {
            Ok(value) => println!("{value}"),
            Err(e) => program.pretty_print_error(e),
        }
    }
}
