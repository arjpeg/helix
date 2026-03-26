use thiserror::Error;

use crate::lexer::token::Token;

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

    #[error("invalid left-hand side of assignment")]
    InvalidAssignmentLhs,
}
