use thiserror::Error;

use crate::{
    token::{Operator, Span, Token, UnaryOperator},
    value::ValueKind,
};

/// An wrapper over Result to be specific to Helix errors
pub type Result<T> = std::result::Result<T, Error>;

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
    #[error(transparent)]
    Parser(#[from] ParserError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

/// An error that occurred during tokenization.
#[derive(Error, Debug, Clone)]
pub enum LexerError {
    #[error("encountered an unknown symbol '{0}'")]
    UnknownSymbol(String),
    #[error("encountered a malformed number '{0}'")]
    MalformedNumber(String),
}

/// An error that occurred during the generation of the AST.
#[derive(Error, Debug, Clone)]
pub enum ParserError {
    #[error("'{0}' is not a valid unary operator")]
    InvalidUnaryOperator(Operator),
    #[error("found unexpected token '{0}'")]
    UnexpectedToken(Token),
}

/// An error that occured during the runtime of the program.
#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    #[error("cannot apply binary operator '{operator}' between values of kind {} and {}", lhs.name(), rhs.name())]
    InvalidBinaryOperation {
        lhs: ValueKind,
        operator: Operator,
        rhs: ValueKind,
    },
    #[error("cannot apply unary operator '{operator}' to a value of kind {}", operand.name())]
    InvalidUnaryOperation {
        operand: ValueKind,
        operator: UnaryOperator,
    },
}
