use std::{
    fmt::{Display, Write},
    ops::Range,
};

/// A token within the source code, representing a literal, operator, or keyword.
#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// The kind of a token.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    /// An integer literal
    Integer(i64),
    /// A floating point literal
    Float(f64),

    /// Any sort of operator
    Operator(Operator),

    /// Any form of whitespace (spaces, tabs, newlines).
    /// Only used for lexing, and is discarded by the lexer.
    Whitespace,
}

/// An operator in the source code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    /// The plus operator (`+`)
    Plus,
    /// The minus operator (`-`)
    Minus,
    /// The multiplication operator (`*`)
    Multiply,
    /// The division operator (`/`)
    Divide,
}

/// A unary operator. Isn't directly created during
/// tokenization, but is used during parsing/interpretation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    /// The plus operator (`+`)
    Plus,
    /// The minus operator (`-`)
    Minus,
}

/// A range within some source code in a file.
#[derive(Debug, Clone, Copy)]
pub struct Span {
    /// The beginning of the span (inclusive).
    pub start: usize,
    /// The end of the span (exclusive).
    pub end: usize,

    /// The source file that this span is in (its index).
    pub source: usize,
}

impl Token {
    /// Create a new token with a given kind and span.
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Span {
    /// Create a new span with a given start and end.
    pub const fn new(Range { start, end }: Range<usize>, source: usize) -> Self {
        Self { start, end, source }
    }
}

impl Operator {
    pub fn from_char(c: char) -> Option<Self> {
        Some(match c {
            '+' => Self::Plus,
            '-' => Self::Minus,
            '*' => Self::Multiply,
            '/' => Self::Divide,
            _ => return None,
        })
    }

    pub fn from_token_kind(kind: TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Operator(op) => Some(op),
            _ => None,
        }
    }
}

impl UnaryOperator {
    pub fn from_operator(op: Operator) -> Option<Self> {
        Some(match op {
            Operator::Plus => Self::Plus,
            Operator::Minus => Self::Minus,
            _ => return None,
        })
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::Plus => '+',
            Self::Minus => '-',
            Self::Multiply => '*',
            Self::Divide => '/',
        })
    }
}
