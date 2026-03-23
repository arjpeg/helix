use clap::Parser;
use helix::{error, interpreter::value::Value, run_program, run_repl, source::Source};
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

    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("".into()),
        ..Default::default()
    };

    loop {
        let Ok(Signal::Success(line)) = line_editor.read_line(&prompt) else {
            break;
        };

        let content: &'static str = Box::leak(line.trim().to_string().into_boxed_str());
        let source = Source {
            content,
            path: Path::new("<repl>"),
        };

        match run_repl(source) {
            Ok(Some(value)) if value != Value::Unit => println!("{value}"),
            Ok(_) => {}
            Err(e) => error::print_error(e),
        }
    }
}
