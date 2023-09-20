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

    /// An invalid unary expression was encountered.
    InvalidUnaryExpression {
        /// The operator that was used.
        operator: OperatorKind,

        /// The expression that was used.
        expr: Value,

        ///The span of the unary expression.
        span: Span,
    },

    /// A division by zero occurred.
    DivisionByZero {
        ///The span of the binary expression.
        span: Span,
    },

    /// An attempt to access a variable that does not exist occurred.
    UndefinedVariable {
        /// The name of the variable that does not exist.
        name: String,

        /// The span of the variable.
        span: Span,
    },

    /// Found a break statement, used for exiting loops. If this is found
    /// outside of a loop, then an error is thrown.
    Break {
        /// The span of the break statement.
        span: Span,
    },

    /// Found a continue statement, used for skipping the rest of the loop
    /// and going to the next iteration. If this is found outside of a loop,
    /// then an error is thrown.
    Continue {
        /// The span of the continue statement.
        span: Span,
    },
}
