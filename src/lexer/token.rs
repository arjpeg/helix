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
    pub fn new(span: Span, token_type: TokenKind) -> Token {
        Token {
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

    // Any whitespace characters (ie. space, tab, newline, etc)
    Whitespace,

    /// A command used during the REPL.
    Command(CommandType),

    // An operator, such as +, -, *, /, etc.
    Operator(OperatorKind),

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
    Quit,
    Help,
}
impl CommandType {
    pub fn get_command(lexeme: &str) -> Option<CommandType> {
        match lexeme {
            "quit" | "q" => Some(CommandType::Quit),
            "help" | "h" => Some(CommandType::Help),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeywordKind {
    Let,
    If,
    Else,
}

impl KeywordKind {
    pub fn get_keyword(lexeme: &str) -> Option<KeywordKind> {
        match lexeme {
            "let" => Some(KeywordKind::Let),
            "if" => Some(KeywordKind::If),
            "else" => Some(KeywordKind::Else),
            _ => None,
        }
    }
}
