use crate::{lexer, parser};

/// An enum that wraps all the errors that can occur during any
/// phase of the interpreter.
#[derive(Debug, Clone)]
pub enum Error {
    /// An error that occurred during lexing.
    LexerError(lexer::error::LexerError),
    /// An error that occurred during parsing.
    ParserError(parser::error::ParserError),
}
