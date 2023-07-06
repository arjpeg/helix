use crate::lexer::{span::Span, token::OperatorKind};

use super::data::Value;

/// An error that occured during the interpretation of the AST.
#[derive(Debug, Clone)]
pub enum InterpreterError {
    /// The left hand side and right hand side of a binary expression
    /// cannot be combined through the given operator.
    InvalidBinaryExpression {
        /// The operator that was used.
        operator: OperatorKind,

        /// The left hand side of the binary expression.
        lhs: Value,

        /// The right hand side of the binary expression.
        rhs: Value,

        ///The span of the binary expression.
        span: Span,
    },

    /// A division by zero occurred.
    DivisionByZero {
        ///The span of the binary expression.
        span: Span,
    },
}
