use std::ops::Range;

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

    /// Any form of whitespace (spaces, tabs, newlines).
    /// Only used for lexing, and is discarded by the lexer.
    Whitespace,
}

/// A range within some source code in a file.
#[derive(Debug, Clone, Copy)]
pub struct Span {
    /// The beginning of the span (inclusive).
    pub start: usize,
    /// The end of the span (exclusive).
    pub end: usize,

    /// The source file that this span is in (it's index).
    pub source: usize,
}

impl Token {
    /// Create a new token with a given kind and span.
    pub fn new(kind: TokenKind, span: Span) -> Token {
        Token { kind, span }
    }
}

impl Span {
    /// Create a new span with a given start and end.
    pub fn new(Range { start, end }: Range<usize>, source: usize) -> Span {
        Span { start, end, source }
    }
}
