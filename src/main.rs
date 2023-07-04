mod errors;
mod input;
mod lexer;
mod parser;

use errors::Error;
use lexer::{error::LexerError, Lexer};
use owo_colors::OwoColorize;

use crate::{
    lexer::{
        span::Span,
        token::{CommandType, TokenKind},
    },
    parser::{error::ParserError, Parser},
};

fn run(code: &String) -> Result<(), Error> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.lex().map_err(|e| Error::LexerError(e))?;

    if tokens.is_empty() {
        return Ok(());
    }

    if let TokenKind::Command(command) = tokens[0].token_kind {
        handle_command(command);
        return Ok(());
    }

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| Error::ParserError(e))?;

    println!("{:#?}", ast);

    Ok(())
}

fn main() {
    input::print_intro();

    loop {
        let input = input::get_input();
        let result = run(&input);

        if let Err(err) = result {
            format_error(input, err);
        }
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

fn format_error(input: String, error: Error) {
    let (message, range) = match error {
        // Lexer errors
        Error::LexerError(error) => match error {
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
        },
        // Parser errors
        Error::ParserError(error) => match error {
            ParserError::UnexpectedToken { found, expected } => (
                format!(
                    "Expected {}, but found a token of kind {:?}",
                    expected, found.token_kind
                ),
                found.span,
            ),
            ParserError::UnexpectedEof { expected } => (
                format!(
                    "Expected {}, but unexpectedly reached the end of file",
                    expected
                ),
                Span::new(input.len() - 1, input.len()),
            ),
        },
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
