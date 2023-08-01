mod errors;
mod input;
mod interpreter;
mod lexer;
mod parser;

use std::rc::Rc;

use errors::Error;
use interpreter::Interpreter;
use lexer::{error::LexerError, Lexer};
use owo_colors::OwoColorize;

use interpreter::data::ValueKind;

use crate::{
    interpreter::error::InterpreterError,
    lexer::token::{CommandType, TokenKind},
    parser::{error::ParserError, Parser},
};

fn run(
    code: &str,
    interpreter: &mut Interpreter,
    in_repl: bool,
    filename: Rc<str>,
) -> Result<(), Error> {
    let mut lexer = Lexer::new(code, Rc::clone(&filename));
    let tokens = lexer.lex()?;

    if tokens.is_empty() {
        return Ok(());
    }

    if let TokenKind::Command(command) = tokens[0].token_kind {
        handle_command(command);
        return Ok(());
    }

    let mut parser = Parser::new(tokens, Rc::clone(&filename));
    let ast = parser.parse()?;

    let result = interpreter.start(ast)?;

    // if the result isn't Null, print it
    if in_repl {
        match result.kind {
            ValueKind::Null => {}
            _ => println!("{}", result.kind),
        }
    }

    Ok(())
}

fn repl() {
    input::print_intro();

    let mut interpreter = Interpreter::new();
    let filename = Rc::from("stdin");

    loop {
        let input = input::get_input();
        let result = run(&input, &mut interpreter, true, Rc::clone(&filename));

        if let Err(err) = result {
            format_error(input, err);
        }
    }
}

fn main() {
    // If there are any arguments, run the file
    if std::env::args().len() > 1 {
        let mut interpreter = Interpreter::new();

        for file in std::env::args().skip(1) {
            let code = std::fs::read_to_string(&file).unwrap();
            let filename = Rc::from(file);
            let result = run(&code, &mut interpreter, false, filename);

            if let Err(err) = result {
                format_error(code, err);
            }
        }

        return;
    }

    repl();
}

fn handle_command(command: CommandType) {
    match command {
        CommandType::Quit => {
            std::process::exit(0);
        }

        CommandType::Help => {
            println!("{} ({}):", "Help".blue().bold(), "Helix v0.1.0".dimmed());
            println!("  {} - Quit the REPL", "#quit".cyan().bold());
            println!("  {} - Show this message", "#help".cyan().bold());

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
        Error::Lexer(error) => match error {
            LexerError::TooManyDots { range } => (
                "A number cannot contain more than one decimal place.".to_string(),
                range,
            ),
            LexerError::UnknownSymbol { range } => {
                (format!("Unknown symbol '{}'", &input[range.clone()]), range)
            }
            LexerError::UnknownCommand { range } => (
                format!("Unknown command '{}'", &input[range.clone()]),
                range,
            ),
            LexerError::UnterminatedString { range } => {
                ("Unterminated string literal".to_string(), range)
            }
        },
        // Parser errors
        Error::Parser(error) => match error {
            ParserError::UnexpectedToken { found, expected } => (
                format!(
                    "Expected {}, but found a token of kind {:?}",
                    expected, found.token_kind
                ),
                found.span,
            ),
            ParserError::UnexpectedEof { expected, file } => (
                format!("Expected {}, but found EOF", expected),
                (input.len()..input.len(), file).into(),
            ),

            ParserError::UnexpectedNewline { span, expected } => (
                format!("Unexpectedly found a newline, expected {expected}"),
                span,
            ),
            ParserError::UnmatchedClosingParen { paren } => (
                "Found an unmatched closing parenthesis".to_string(),
                paren.span,
            ),
        },
        // Interpreter errors
        Error::Interpreter(error) => match error {
            InterpreterError::InvalidBinaryExpression {
                operator,
                lhs,
                rhs,
                span,
            } => (
                format!(
                    "Cannot use the operator {:?} between values of type {:?} and {:?}",
                    operator, lhs.kind, rhs.kind,
                ),
                span,
            ),
            InterpreterError::InvalidUnaryExpression {
                operator,
                expr,
                span,
            } => (
                format!(
                    "Cannot use the operator {:?} on a value of type {:?}",
                    operator, expr.kind
                ),
                span,
            ),

            InterpreterError::DivisionByZero { span } => ("Division by zero".to_string(), span),

            InterpreterError::UndefinedVariable { name, span } => (
                format!("Can't find variable '{}' in the current scope", name),
                span,
            ),
        },
    };

    // Get the line in which the error occurred
    let line_num = input[..range.start].matches('\n').count() + 1;

    let line = input
        .lines()
        .nth(if line_num != 0 { line_num - 1 } else { 0 })
        .expect("Line number out of range");

    // Get the range of the error in the line
    let line_start_index = input[..range.start].rfind('\n').unwrap_or(0) + 1;

    let file = range.file;
    let range = (if range.start >= line_start_index {
        range.start - line_start_index + 1
    } else {
        0
    })..range.end + 1 - line_start_index;

    let location = format!("{}:{}", file, line_num);

    eprintln!("{}: {}", "Error".red().bold(), message.bold());
    eprintln!(" {}  {}", location.dimmed(), line.bold());
    eprintln!(
        " {}  {}",
        " ".repeat(location.len() + range.start),
        "^".repeat(range.len())
    );
}
