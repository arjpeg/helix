/// The smallest lexical unit in the source code.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Token {
    /// An integer literal.
    Int(i64),
    /// A keyword.
    Keyword(Keyword),
    /// An unknown symbol (usually represents a variable name).
    Symbol(&'static str),
    /// Any operator.
    Operator(OpKind),
    /// Any grouping symbol.
    Grouping(Grouping),
    /// A semicolon, marking the end of a statement.
    Semicolon,

    /// The end of file (EOF).
    Eof,
}

/// A literal operator in the source code.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

/// Any literal grouping symbol in the source code.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Grouping {
    /// A '(' parenthesis.
    OpeningParen,
    /// A ')' parenthesis.
    ClosingParen,
}

/// Any reserved keyword in the source code.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Keyword {
    /// The 'and' keyword.
    And,
    /// The 'or' keyword.
    Or,
    /// The 'true' keyword.
    True,
    /// The 'false' keyword.
    False,
}

pub trait CharTokenExt {
    /// Returns `true` if the provided character is the beginning of a operator sequence.
    fn is_operator_start(&self) -> bool;
    /// Returns `true` if the provided character is a valid grouping symbol.
    fn is_grouping(&self) -> bool;
}

impl CharTokenExt for char {
    fn is_operator_start(&self) -> bool {
        matches!(self, '+' | '-' | '*' | '/' | '=' | '!' | '>' | '<')
    }

    fn is_grouping(&self) -> bool {
        matches!(self, '(' | ')')
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

impl TryFrom<char> for Grouping {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '(' => Self::OpeningParen,
            ')' => Self::ClosingParen,
            _ => return Err(()),
        })
    }
}

impl TryFrom<&str> for Keyword {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "and" => Self::And,
            "or" => Self::Or,
            "true" => Self::True,
            "false" => Self::False,
            _ => return Err(()),
        })
    }
}

impl From<Keyword> for Token {
    fn from(value: Keyword) -> Self {
        Self::Keyword(value)
    }
}

impl From<OpKind> for Token {
    fn from(value: OpKind) -> Self {
        Self::Operator(value)
    }
}

impl From<Grouping> for Token {
    fn from(value: Grouping) -> Self {
        Self::Grouping(value)
    }
}
