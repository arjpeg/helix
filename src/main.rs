mod errors;
mod input;
mod interpreter;
mod lexer;
mod parser;

use std::rc::Rc;

use errors::Error;
use input::Input;
use interpreter::Interpreter;
use lexer::{error::LexerError, Lexer};
use owo_colors::OwoColorize;

use interpreter::data::ValueKind;

use crate::{
    interpreter::error::InterpreterError,
    lexer::token::{CommandType, TokenKind},
    parser::{error::ParserError, Parser},
};

use rustc_version::version as rustc_version;

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

    let mut input = Input::new();

    let mut interpreter = Interpreter::new();
    let filename = Rc::from("stdin");

    loop {
        let input = input.get_line();
        let result = run(&input, &mut interpreter, true, Rc::clone(&filename));

        if let Err(err) = result {
            format_error(input, err);
        }
    }
}

fn handle_command(command: CommandType) {
    let version = env!("CARGO_PKG_VERSION");

    match command {
        CommandType::Quit => {
            std::process::exit(0);
        }

        CommandType::Help => {
            println!("{} (Helix v{}):", "Help".blue().bold(), version.dimmed());
            println!("  {} - Quit the REPL", "#quit".cyan().bold());
            println!("  {} - Show this message", "#help".cyan().bold());
            println!("  {} - Show the current version", "#version".cyan().bold());
            println!(
                "  {} - Show the licence information",
                "#licence".cyan().bold()
            );

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

        CommandType::Version => {
            println!("{}: {}", "Helix Version".bold(), version.blue().bold());
            println!(
                "  {} {}",
                "Compiled with Rust Version".dimmed(),
                rustc_version().expect("Unknown rust version").cyan()
            );
        }

        CommandType::Licence => {
            println!("{}:", "Licence".blue().bold());
            println!(
                "  Helix is licenced under the {}.",
                "MIT licence".bold().cyan()
            );
            println!(
                "  For more information, visit {}",
                "https://helix-lang.org/licence".cyan()
            );
        }
    }
}

fn format_error(input: String, error: Error) {
    use InterpreterError as IE;
    use LexerError as LE;
    use ParserError as PE;

    let (message, range) = match error {
        // Lexer errors
        Error::Lexer(error) => match error {
            LE::TooManyDots { range } => (
                "A number cannot contain more than one decimal place.".to_string(),
                range,
            ),
            LE::UnknownSymbol { range } => {
                (format!("Unknown symbol '{}'", &input[range.clone()]), range)
            }
            LE::UnknownCommand { range } => (
                format!("Unknown command '{}'", &input[range.clone()]),
                range,
            ),
            LE::UnterminatedString { range } => ("Unterminated string literal".to_string(), range),
        },
        // Parser errors
        Error::Parser(error) => match error {
            PE::UnexpectedToken { found, expected } => (
                format!(
                    "Expected {}, but found a token of kind {:?}",
                    expected, found.token_kind
                ),
                found.span,
            ),
            PE::UnexpectedEof { expected, file } => (
                format!("Expected {}, but found EOF", expected),
                (input.len()..input.len(), file).into(),
            ),

            PE::UnexpectedNewline { span, expected } => (
                format!("Unexpectedly found a newline, expected {expected}"),
                span,
            ),
            PE::UnmatchedClosingParen { paren } => (
                "Found an unmatched closing parenthesis".to_string(),
                paren.span,
            ),
            PE::InvalidAssignmentTarget { found } => (
                format!(
                    "Found an invalid left hand side during variable assignment (of kind, '{}')",
                    found.kind
                ),
                found.span,
            ),
        },

        // Interpreter errors
        Error::Interpreter(error) => match error {
            IE::InvalidBinaryExpression {
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
            IE::InvalidUnaryExpression {
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

            IE::DivisionByZero { span } => ("Division by zero".to_string(), span),

            IE::UndefinedVariable { name, span } => (
                format!("Can't find variable '{}' in the current scope", name),
                span,
            ),

            IE::Break { span } => (
                "Found a break statement outside of a loop".to_string(),
                span,
            ),

            IE::Continue { span } => (
                "Found a continue statement outside of a loop".to_string(),
                span,
            ),
        },
    };

    // Get the line in which the error occurred
    let line_num = input[..range.start].matches('\n').count();

    println!("range {range:?}");
    println!("{}", &input[..range.start]);
    println!("line num {line_num}");

    let line = input
        .lines()
        .nth(line_num)
        .expect("Line number out of range");

    println!("line {line}");

    // Get the range of the error in the line
    let line_start_index = input[..range.start].rfind('\n').unwrap_or(0) + 1;

    let file = range.file;
    let range = (if range.start >= line_start_index {
        range.start - line_start_index + 1
    } else {
        0
    })..range.end + 1 - line_start_index;

    let location = format!("{}:{}", file, line_num);

    println!("{range:?}");

    eprintln!("{}: {}", "Error".red().bold(), message.bold());
    eprintln!(" {}  {}", location.dimmed(), line.bold());
    eprintln!(
        " {}  {}",
        " ".repeat(location.len() + range.start),
        "^".repeat(range.len())
    );
}
