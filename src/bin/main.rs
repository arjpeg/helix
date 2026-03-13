use clap::Parser;
use helix::{error, run_program, run_repl, source::Source};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "helix")]
struct Cli {
    /// Source file to execute (omit to start REPL)
    file: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    match cli.file {
        Some(path) => {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: {e}");
                    return;
                }
            };

            let source = Source {
                content: Box::leak(content.into_boxed_str()),
                path: Box::leak(path.into_boxed_path()),
            };

            if let Err(e) = run_program(source) {
                error::print_error(e);
            }
        }

        None => repl(),
    }
}

fn repl() {
    println!("helix REPL (ctrl+c to exit)");
    let stdin = io::stdin();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() || line.is_empty() {
            break;
        }

        let content: &'static str = Box::leak(line.trim().to_string().into_boxed_str());
        let source = Source {
            content,
            path: Path::new("<repl>"),
        };

        match run_repl(source) {
            Ok(Some(value)) => println!("{value}"),
            Ok(None) => {}
            Err(e) => error::print_error(e),
        }
    }
}
