use std::io::{self, Write};

use helix::program::Program;

fn main() {
    match std::env::args().nth(1) {
        Some(_filename) => todo!(),
        None => repl(),
    }
}

fn repl() {
    let mut program = Program::new();

    loop {
        //let mut line = String::new();
        // print!("helix > ");

        // io::stdout().flush().unwrap();
        // io::stdin().read_line(&mut line).unwrap();
        let line = "2 + 123".to_string();

        let main = program.add_source("<stdin>".to_string(), line);

        match program.run(main) {
            Ok(value) => println!("{value}"),
            Err(e) => program.pretty_print_error(e),
        }
        break;
    }
}
