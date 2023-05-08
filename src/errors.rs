use crate::tokens::{Span, TokenKind};

/// An error that can occur while tokenizing.
#[derive(Debug, PartialEq)]
pub enum TokenizerError {
    /// An unexpected character was encountered.
    /// Keeps track of the byte position of the unexpected character.
    UnexpectedIdentifier {
        /// The byte position of the unexpected character.
        span: Span,

        /// The unexpected identifier.
        identifier: String,
    },

    /// There were too many decimal points in a number literal.
    TooManyDecimalPoints {
        /// The byte position of the unexpected character.
        span: Span,
    },
}

#[derive(Debug)]
pub enum ParserError {
    /// When the parser expected a binary operator
    /// (`+`, `-`, `*`, `/`) but found something else.
    ExpectedBinaryOperator(Span),

    /// When the parser expected a new expression
    /// (`-`, `(`, or a number literal) but found something else.
    ExpectedNewExpression(Span),

    /// There were too few closing parentheses.
    UnclosedParenthesis(Span),

    /// There were too many closing parentheses.
    UnmatchedClosingParenthesis(Span),

    /// When the parser expected a token of a certain kind
    /// but found something else.
    UnexpectedToken {
        /// The byte position of the unexpected token.
        span: Span,

        /// The expected token kind.
        expected: String,

        /// The actual token kind.
        found: TokenKind,
    },
    /// When the parser expected a token of a certain kind
    /// but found an unexpected end of input.
    UnexpectedEof(Span),
}

pub enum HelixError {
    TokenizerError(TokenizerError),
    ParserError(ParserError),
}
