/// The smallest lexical unit in the source code.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    /// An integer literal.
    Integer(u64),
    /// A symbol (usually represents a variable name).
    Symbol(&'static str),
    /// Any operator.
    Operator(OpKind),
}

/// A literal operator in the source code.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum OpKind {
    /// The '+' operator.
    Plus,
    /// The '-' operator.
    Minus,
    /// The '*' operator.
    Star,
    /// The '/' operator.
    Slash,

    /// The '!' operator.
    Bang,

    /// The '=' operator.
    Assign,

    /// The '!=' operator.
    NotEquals,
    /// The '==' operator.
    Equals,

    /// The '>' operator.
    GreaterThan,
    /// The '>=' operator.
    GreaterThanEquals,
    /// The '<' operator.
    LessThan,
    /// The '<=' operator.
    LessThanEquals,
}

pub trait CharTokenExt {
    /// Returns `true` if the provided character is the beginning of a operator sequence.
    fn is_operator_start(&self) -> bool;
}

impl CharTokenExt for char {
    fn is_operator_start(&self) -> bool {
        matches!(self, '+' | '-' | '*' | '/' | '=' | '!' | '>' | '<')
    }
}

impl OpKind {
    /// Returns the byte length of this operator in text format.
    pub fn len(&self) -> usize {
        match self {
            Self::Equals | Self::NotEquals | Self::GreaterThanEquals | Self::LessThanEquals => 2,
            _ => 1,
        }
    }
}

impl TryFrom<(char, Option<char>)> for OpKind {
    type Error = ();

    fn try_from(value: (char, Option<char>)) -> Result<Self, Self::Error> {
        Ok(match value {
            ('=', Some('=')) => Self::Equals,
            ('!', Some('=')) => Self::NotEquals,
            ('>', Some('=')) => Self::GreaterThanEquals,
            ('<', Some('=')) => Self::LessThanEquals,
            ('+', _) => Self::Plus,
            ('-', _) => Self::Minus,
            ('*', _) => Self::Star,
            ('/', _) => Self::Slash,
            ('>', _) => Self::GreaterThan,
            ('<', _) => Self::LessThan,
            ('!', _) => Self::Bang,
            ('=', _) => Self::Assign,
            _ => return Err(()),
        })
    }
}
