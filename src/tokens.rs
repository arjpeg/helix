//! Contains the definitions for all the tokens used in the language.

use std::ops::{Index, Range};

/// A Span to repesent the location of a token in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The starting byte of the token.
    pub start: usize,
    /// The ending byte of the token.
    pub end: usize,
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl Index<Span> for str {
    type Output = Self;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}

/// Used to represent an operator.
#[derive(Debug, PartialEq, Clone)]
pub enum OperatorKind {
    /// '+'
    Plus,
    /// '-'
    Minus,
    /// '*'
    Star,
    /// '/'
    Slash,
}

/// The type of a token.
#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    /// Whitespace tokens (ie. ' ', '\n', '\t', etc.)
    Whitespace,

    /// A number literal (represented as a f64)
    Number(f64),

    /// An operator (ie. '+', '-', '*', '/')
    Operator(OperatorKind),

    /// Parenthesis (ie. '(', ')')
    OpenParenthesis,
    CloseParenthesis,

    /// EOF token.
    Eof,
}

/// The actual token.
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
