use thiserror::Error;

use crate::{lexer::token::Token, source::Spanned};

/// A type alias for the result of an operation that occured during parsing.
pub type Result<T, E = Spanned<ParsingError>> = std::result::Result<T, E>;

/// An error that occured during the parsing process.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ParsingError {
    #[error("unexpectedly reached the end of file")]
    UnexpectedEof,
    #[error("expected to find {expected}, but found a token of type `{found:?}`")]
    UnexpectedToken {
        expected: &'static str,
        found: Token,
    },
}
