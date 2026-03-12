use crate::{lexer::token::OpKind, source::Spanned};

/// A statement in the AST.
#[derive(Debug, Clone)]
pub enum Statement {
    /// An expression statement that wasn't saved.
    Expression {
        /// The expression to evaluate.
        expr: Expression,
    },
}

/// An expression in the AST.
#[derive(Debug, Clone)]
pub enum Expression {
    /// An integer literal.
    Integer(i64),

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
