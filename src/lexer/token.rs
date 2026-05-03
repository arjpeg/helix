/// The smallest lexical unit in the source code.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Token {
    /// An integer literal.
    Int(i64),
    /// A string literal.
    String(&'static str),
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
    /// A comma, delineating multiple items in a list.
    Comma,

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
    /// A '{' curly bracket.
    OpeningCurly,
    /// A '}' curly bracket.
    ClosingCurly,
}

/// Any reserved keyword in the source code.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Keyword {
    /// The 'and' keyword checks if two boolean values are both true.
    And,
    /// The 'or' keyword checks if either of two boolean values are true.
    Or,

    /// The 'true' keyword, acting as the `true` boolean literal.
    True,
    /// The 'false' keyword, acting as the `false` boolean literal.
    False,

    /// The 'print' keyword outputs the value of the subsequent expression.
    Print,

    /// The 'let' keyword binds a symbol (known as the binding) to a value.
    Let,

    /// The 'assert' keyword runs a runtime assertion to ensure the subsequent expression is truthy.
    Assert,

    /// The 'if' keyword begins a standard if expression to run code iff a condition is truthy.
    If,
    /// The 'else' keyword executes code if its corresponding if predicated evaluated to a non
    /// truthy value.
    Else,

    /// The 'while' keyword repeatedly executes a body while some predicate is truthy.
    While,
    /// The 'break' keyword exits the innermost running loop.
    Break,

    /// The 'fn' keyword begins a new function declaration if followed by a name, else begins a lambda expression.
    Fn,
    /// The 'return' escapes from the current functions scope, returning a value to the caller.
    Return,
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
        matches!(self, '(' | ')' | '{' | '}')
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
            '{' => Self::OpeningCurly,
            '}' => Self::ClosingCurly,
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
            "print" => Self::Print,
            "let" => Self::Let,
            "assert" => Self::Assert,
            "if" => Self::If,
            "else" => Self::Else,
            "while" => Self::While,
            "break" => Self::Break,
            "fn" => Self::Fn,
            "return" => Self::Return,
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
