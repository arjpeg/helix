use crate::{
    lexer::token::{Keyword, OpKind, Token},
    source::Spanned,
};

/// A statement in the AST.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A complete helix program.
    Program {
        /// The list of statements to execute, in order.
        stmts: Vec<Spanned<Statement>>,
    },

    /// A [Statement::Program] that can optionally return a tail value.
    ReplInput {
        /// The list of statements to execute, in order.
        stmts: Vec<Spanned<Statement>>,
        /// The optional tail expression of this input (what it returns).
        tail: Option<Box<Spanned<Expression>>>,
    },

    /// A statement that prints the result of the [`Expression`] to stdout.
    Print(Spanned<Expression>),

    /// A declaration of a variable binding.
    Declaration {
        /// The name of the binding to declare.
        symbol: &'static str,
        /// The value to assign.
        value: Spanned<Expression>,
    },

    /// Asserts that the given [`Expression`] evaluates to `true`.
    Assert(Spanned<Expression>),

    /// A standalone expression.
    Expression {
        /// The expression to evaluate.
        expr: Expression,
        /// Whether or not the expression was terminated with a semicolon (is a tail).
        has_semicolon: bool,
    },
}

/// An expression in the AST.
#[derive(Debug, Clone)]
pub enum Expression {
    /// An integer literal.
    Integer(i64),
    /// A boolean literal.
    Boolean(bool),

    /// A reference to a variable.
    Variable {
        /// The symbol of the binding.
        symbol: &'static str,
    },

    /// A (infix) binary operation between two other [`Expression`]s.
    BinaryOperation {
        /// The left hand side operand.
        lhs: Box<Spanned<Expression>>,
        /// The operator applied between.
        operator: BinaryOp,
        /// The right hand side operand.
        rhs: Box<Spanned<Expression>>,
    },

    /// A unary operation acting on one [`Expression`].
    UnaryOperation {
        /// The unary operator acting on the `operand`.
        operator: UnaryOp,
        /// The value being acted on.
        operand: Box<Spanned<Expression>>,
    },

    /// A grouping of statements, delimited by a pair of '{' and '}'.
    Block {
        /// The list of statements to execute, in order.
        stmts: Vec<Spanned<Statement>>,
        /// The optional tail expression of this block (what it returns).
        tail: Option<Box<Spanned<Expression>>>,
    },
}

/// A binary operator applied between two [`Expression`]s.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BinaryOp {
    /// The '+' operator.
    Plus,
    /// The '-' operator.
    Minus,
    /// The '*' operator.
    Star,
    /// The '/' operator.
    Slash,

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

    /// The 'and' operator.
    And,
    /// The 'or' operator.
    Or,
}

/// A unary operator applied on an [`Expression`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum UnaryOp {
    /// The '+' operator.
    Plus,
    /// The '-' operator.
    Minus,
    /// The '!' operator.
    Bang,
}

impl TryFrom<OpKind> for UnaryOp {
    type Error = ();

    fn try_from(value: OpKind) -> Result<Self, Self::Error> {
        Ok(match value {
            OpKind::Plus => Self::Plus,
            OpKind::Minus => Self::Minus,
            OpKind::Bang => Self::Bang,
            _ => return Err(()),
        })
    }
}

impl TryFrom<Token> for BinaryOp {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Operator(op) => Self::try_from(op),
            Token::Keyword(keyword) => Self::try_from(keyword),
            _ => return Err(()),
        }
    }
}

impl TryFrom<OpKind> for BinaryOp {
    type Error = ();

    fn try_from(value: OpKind) -> Result<Self, Self::Error> {
        Ok(match value {
            OpKind::Plus => Self::Plus,
            OpKind::Minus => Self::Minus,
            OpKind::Star => Self::Star,
            OpKind::Slash => Self::Slash,
            OpKind::NotEquals => Self::NotEquals,
            OpKind::Equals => Self::Equals,
            OpKind::GreaterThan => Self::GreaterThan,
            OpKind::GreaterThanEquals => Self::GreaterThanEquals,
            OpKind::LessThan => Self::LessThan,
            OpKind::LessThanEquals => Self::LessThanEquals,
            _ => return Err(()),
        })
    }
}
impl TryFrom<Keyword> for BinaryOp {
    type Error = ();

    fn try_from(value: Keyword) -> Result<Self, Self::Error> {
        Ok(match value {
            Keyword::And => Self::And,
            Keyword::Or => Self::Or,
            _ => return Err(()),
        })
    }
}
