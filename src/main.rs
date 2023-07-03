mod input;
mod lexer;

use lexer::{error::LexerError, Lexer};
use owo_colors::OwoColorize;

use crate::lexer::token::{CommandType, TokenKind};

fn main() {
    input::print_intro();

    loop {
        let input = input::get_input();

        let mut lexer = Lexer::new(&input);
        let tokens = lexer.lex();

        if let Err(error) = tokens {
            format_error(input, error);
            continue;
        }

        let tokens = tokens.unwrap();

        if let TokenKind::Command(command) = tokens[0].token_kind {
            handle_command(command);
            continue;
        }

        println!("{:#?}", tokens)
    }
}

fn handle_command(command: CommandType) {
    match command {
        CommandType::Quit => {
            std::process::exit(0);
        }

        CommandType::Help => {
            println!("{} ({}):", "Help".blue().bold(), "Helix v0.1.0".dimmed());
            println!("  {} - {}", "#quit".cyan().bold(), "Quit the REPL");
            println!("  {} - {}", "#help".cyan().bold(), "Show this message");

            println!();
            println!(
                "Press {} to abort the current expression, {} to exit.",
                "Ctrl+C".blue(),
                "Ctrl+D".blue()
            );
            println!(
                "For more information about Helix, visit {}.",
                "https://helix-lang.org".green()
            );
        }
    }
}

fn format_error(input: String, error: LexerError) {
    let (message, range) = match error {
        LexerError::TooManyDots { range } => (
            format!("A number cannot contain more than one decimal place."),
            range,
        ),
        LexerError::UnknownSymbol { range } => {
            (format!("Unknown symbol '{}'", &input[range]), range)
        }

        LexerError::UnknownCommand { range } => {
            (format!("Unknown command '{}'", &input[range]), range)
        }
    };

    // Get the line in which the error occurred
    let line_num = input[..range.start].matches('\n').count() + 1;
    let line = input
        .lines()
        .nth(line_num - 1)
        .expect("Line number out of range");

    let location = format!("{}:{}", "stdin", line_num);

    eprintln!("{}: {}", "Error".red().bold(), message.bold());
    eprintln!(" {}  {}", location.dimmed(), line.bold());
    eprintln!(
        " {}  {}",
        " ".repeat(location.len() + range.start),
        "^".repeat(range.len())
    );
}
