use super::span::Span;

/// Struct that represents a token.
/// A token contains a lexeme, and a token type.
/// A lexeme is the actual string that was matched,
/// while the token type is the type of the token
#[derive(Debug, Clone)]
pub struct Token {
    /// The span / range of the token.
    pub span: Span,
    /// The type of the token.
    pub token_kind: TokenKind,
}

impl Token {
    /// Creates a new token from a lexeme and a token type.
    pub const fn new(span: Span, token_type: TokenKind) -> Token {
        Self {
            span,
            token_kind: token_type,
        }
    }

    #[allow(dead_code)]
    /// Compares the a token to another token.
    pub fn matches(&self, token_type: TokenKind) -> bool {
        self.token_kind == token_type
    }
}

/// Enum that represents the type of a token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // A number literal
    Number(f64),

    // A string literal
    String(String),

    // Any whitespace characters (ie. space, tab, newline, etc)
    Whitespace,

    /// A command used during the REPL.
    Command(CommandType),

    // An operator, such as +, -, *, /, etc.
    Operator(OperatorKind),

    Comma,

    // Parenthesis, ie. (, )
    LeftParen,
    RightParen,

    // Left and right curly braces, ie. {, }
    LeftBrace,
    RightBrace,

    // An identifier, ie. a variable name
    Identifier {
        name: String,
    },

    // A keyword, ie. let, if, else, etc.
    Keyword(KeywordKind),

    // A newline or semicolon
    Newline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatorKind {
    Plus,
    Minus,
    Star,
    Slash,
    Power,

    // The not operator, ie. !
    Bang,

    // Assignment operator, ie. =
    Assign,

    // Comparison operators, ie. ==, !=, <, >, <=, >=
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    Quit,
    Help,
    Version,
    Licence,
}

impl CommandType {
    pub fn get_command(lexeme: &str) -> Option<Self> {
        match lexeme {
            "quit" | "q" => Some(Self::Quit),
            "help" | "h" => Some(Self::Help),
            "version" | "v" => Some(Self::Version),
            "licence" | "license" | "l" => Some(Self::Licence),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
    /// Variable declaration, ie. let _ = 1;
    Let,
    /// If statement, ie. if true { ... }
    If,
    /// Else statement, ie. if true { ... } else { ... }
    Else,
    /// Print statement, ie. print "Hello, world!"
    Print,
    /// Function declaration, ie. fn add(a, b) { ... }
    Function,
    /// While loop, ie. while true { ... }
    While,
    /// Break statement, ie. while true { break };
    Break,
    /// Continue statement, ie. while true { continue };
    Continue,
}

impl KeywordKind {
    pub fn get_keyword(lexeme: &str) -> Option<KeywordKind> {
        match lexeme {
            "let" => Some(Self::Let),
            "if" => Some(Self::If),
            "else" => Some(Self::Else),
            "print" => Some(Self::Print),
            "while" => Some(Self::While),
            "fn" => Some(Self::Function),
            "break" => Some(Self::Break),
            "continue" => Some(Self::Continue),
            _ => None,
        }
    }
}
