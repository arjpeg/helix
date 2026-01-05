use crate::{
    lexer::token::{OpKind, UnaryOp},
    source::Spanned,
};

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
    Integer(u64),

    /// A (infix) binary operation between two other [`Expression`]s.
    BinaryOperation {
        /// The left hand side operand.
        lhs: Box<Spanned<Expression>>,
        /// The operator applied between.
        operator: OpKind,
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
