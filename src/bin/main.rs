use clap::Parser;
use helix::{engine::Engine, error, source::Source};
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
    let mut engine = Engine::new();

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

            match engine.register(source) {
                Ok(s) => engine.excecute(s),
                Err(e) => error::print_error(e),
            }
        }

        None => repl(&mut engine),
    }
}

fn repl(engine: &mut Engine) {
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

        match engine.register(source) {
            Ok(s) => engine.excecute(s),
            Err(e) => error::print_error(e),
        }
    }
}
