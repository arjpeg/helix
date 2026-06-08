use clap::Parser;
use helix::{Engine, error, interpreter::value::Value, source::SourceMap};
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
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

            let source = SourceMap::add(content.leak(), path.leak());

            if let Err(errors) = engine.register_program(source) {
                for error in errors {
                    error::print_error(error);
                }
                return;
            }

            if let Err(e) = engine.execute(source) {
                error::print_error(e);
            }
        }

        None => repl(),
    }
}

fn repl() {
    println!("helix REPL (ctrl+c to exit)");

    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("".into()),
        ..Default::default()
    };

    let mut engine = Engine::new();

    loop {
        let Ok(Signal::Success(line)) = line_editor.read_line(&prompt) else {
            break;
        };

        let source = SourceMap::add(line.leak(), Path::new("<repl>"));

        if let Err(errors) = engine.register_repl(source) {
            for error in errors {
                error::print_error(error);
            }

            continue;
        }

        match engine.execute(source) {
            Ok(Some(value)) if value != Value::Unit => println!("{value}"),
            Ok(_) => {}
            Err(e) => error::print_error(e),
        }
    }
}
