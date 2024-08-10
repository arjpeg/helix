use crate::cursor::Cursor;
use slotmap::DefaultKey;
use std::{fmt::Display, ops::Range, str::Chars};

pub type ASTNode = crate::ast::Node;

/// A token within the source code, representing a literal, operator, or keyword.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// The kind of a token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// An integer literal.
    Integer(i64),
    /// A floating point literal.
    Float(f64),

    /// An identifier.
    Identifier(String),

    /// Any binary operator.
    BinaryOperator(BinaryOperator),
    /// Any unary operator.
    UnaryOperator(UnaryOperator),

    /// A keyword.
    Keyword(Keyword),

    /// Any form of whitespace (spaces, tabs, newlines).
    /// Only used for lexing, and is discarded by the lexer.
    Whitespace,
}

/// A keyword in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    /// The `true` literal
    True,
    /// The `false` literal
    False,
}

/// An operator in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// The plus operator (`+`)
    Plus,
    /// The minus operator (`-`)
    Minus,
    /// The multiplication operator (`*`)
    Multiply,
    /// The division operator (`/`)
    Divide,

    /// The equals operator (`==`)
    Equals,
    /// The not equals operator (`!=`)
    NotEquals,

    /// The less than operator (`<`)
    LessThan,
    /// The less than or equals to operator (`<=`)
    LessThanEquals,

    /// The greater than operator (`>`)
    GreaterThan,
    /// The greater than or equals to operator (`>=`)
    GreaterThanEquals,
}

/// A unary operator. Isn't directly created during
/// tokenization, but is used during parsing/interpretation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// The plus operator (`+`)
    Plus,
    /// The minus operator (`-`)
    Minus,
    /// The not operator (`!`)
    Not,
}

/// A range within some source code in a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The beginning of the span (inclusive).
    pub start: usize,
    /// The end of the span (exclusive).
    pub end: usize,

    /// The key of the source file that this span is in.
    pub source: DefaultKey,
}

impl Token {
    /// Create a new token with a given kind and span.
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Span {
    /// Create a new span with a given start and end.
    pub const fn new(range: Range<usize>, source: DefaultKey) -> Self {
        let Range { start, end } = range;

        Self { start, end, source }
    }
}

impl BinaryOperator {
    pub fn is_operator_start(c: char) -> bool {
        matches!(c, '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/')
    }

    pub fn from_cursor(cursor: &mut Cursor<Chars>) -> Option<Self> {
        Some(match (cursor.advance()?, cursor.peek().copied()) {
            ('+', _) => Self::Plus,
            ('-', _) => Self::Minus,
            ('*', _) => Self::Multiply,
            ('/', _) => Self::Divide,

            ('=', Some('=')) => Self::Equals,
            ('!', Some('=')) => Self::NotEquals,

            ('<', Some('=')) => Self::LessThanEquals,
            ('<', _) => Self::LessThan,

            ('>', Some('=')) => Self::GreaterThanEquals,
            ('>', _) => Self::GreaterThan,

            (_, _) => return None,
        })
    }

    pub fn is_two_char(&self) -> bool {
        matches!(
            self,
            Self::Equals | Self::NotEquals | Self::LessThanEquals | Self::GreaterThanEquals
        )
    }

    pub fn from_token_kind(kind: &TokenKind) -> Option<Self> {
        match kind {
            TokenKind::BinaryOperator(op) => Some(*op),
            _ => None,
        }
    }
}

impl UnaryOperator {
    pub fn from_char(c: char) -> Option<Self> {
        Some(match c {
            '!' => Self::Not,
            '-' => Self::Minus,
            '+' => Self::Plus,
            _ => return None,
        })
    }

    pub fn from_operator(op: BinaryOperator) -> Option<Self> {
        Some(match op {
            BinaryOperator::Plus => Self::Plus,
            BinaryOperator::Minus => Self::Minus,
            _ => return None,
        })
    }
}

impl Keyword {
    pub fn from_ident(ident: &str) -> Option<Self> {
        Some(match ident {
            "true" => Self::True,
            "false" => Self::False,
            _ => return None,
        })
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Equals => "==",
            Self::NotEquals => "!=",
            Self::LessThan => "<",
            Self::LessThanEquals => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanEquals => ">=",
        })
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Not => "!",
        })
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::True => "true",
            Self::False => "false",
        })
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.kind))
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Integer(lit) => lit.to_string(),
            Self::Float(lit) => lit.to_string(),
            Self::Identifier(ident) => ident.clone(),
            Self::BinaryOperator(op) => op.to_string(),
            Self::UnaryOperator(op) => op.to_string(),
            Self::Keyword(keyword) => keyword.to_string(),
            Self::Whitespace => "<whitespace>".to_string(),
        })
    }
}
