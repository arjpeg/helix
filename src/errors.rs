use crate::{interpreter, lexer, parser};

/// An enum that wraps all the errors that can occur during any
/// phase of the interpreter.
#[derive(Debug, Clone)]
pub enum Error {
    /// An error that occurred during lexing.
    Lexer(lexer::error::LexerError),

    /// An error that occurred during parsing.
    Parser(parser::error::ParserError),

    /// An error that occurred during parsing.
    Interpreter(interpreter::error::InterpreterError),
}
