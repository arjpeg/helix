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

impl From<lexer::error::LexerError> for Error {
    fn from(error: lexer::error::LexerError) -> Self {
        Self::Lexer(error)
    }
}

impl From<parser::error::ParserError> for Error {
    fn from(error: parser::error::ParserError) -> Self {
        Self::Parser(error)
    }
}

impl From<interpreter::error::InterpreterError> for Error {
    fn from(error: interpreter::error::InterpreterError) -> Self {
        Self::Interpreter(error)
    }
}
