/// The smallest parsable unit in the source code.
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
}

impl TryFrom<char> for OpKind {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '+' => Self::Plus,
            '-' => Self::Minus,
            '*' => Self::Star,
            '/' => Self::Slash,
            _ => return Err(()),
        })
    }
}
