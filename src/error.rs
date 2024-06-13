use thiserror::Error;

use crate::token::Span;

#[derive(Error, Debug, Clone)]
#[error("{kind}")]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

#[derive(Error, Debug, Clone)]
pub enum ErrorKind {
    #[error(transparent)]
    Lexer(#[from] LexerError),
}

/// An error that occurred during tokenization
#[derive(Error, Debug, Clone)]
pub enum LexerError {
    #[error("encountered an unknown symbol '{0}'")]
    UnknownSymbol(String),
    #[error("encountered a malformed number '{0}'")]
    MalformedNumber(String),
}
