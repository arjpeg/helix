use std::{env, fs};

use owo_colors::OwoColorize;

use helix::program::Program;
use rustyline::DefaultEditor;

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
    let mut rl = DefaultEditor::new().unwrap();
    let mut program = Program::new();

    loop {
        let line = match rl.readline(&format!("{} > ", "helix".green())) {
            Ok(line) => {
                rl.add_history_entry(&line).unwrap();
                line
            }
            Err(_) => break,
        };

        let main = program.add_source("<stdin>".to_string(), line);

        match program.run(main) {
            Ok(value) => println!("{value}"),
            Err(e) => program.pretty_print_error(e),
        }
    }
}
