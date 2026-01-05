use thiserror::Error;

use crate::source::Spanned;

/// A type alias for the result of an operation that occured during tokenization.
pub type Result<T, E = Spanned<TokenizationError>> = std::result::Result<T, E>;

/// An error that occured during the tokenization process.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TokenizationError {
    #[error("encountered an unknown symbol: `{0}`")]
    UnknownSymbol(&'static str),
    #[error("encountered an invalid integer literal: `{0}`")]
    InvalidIntegerLiteral(&'static str),
}
