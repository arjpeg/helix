use thiserror::Error;

use crate::{
    parser::ast::{BinaryOp, UnaryOp},
    source::Spanned,
    vm::r#type::Type,
};

/// A type alias for the result of an operation that occurred during runtime.
pub type Result<T, E = Spanned<RuntimeError>> = std::result::Result<T, E>;

/// An error that occured during the runtime of the VM.
#[derive(Debug, Clone, Error)]
pub enum RuntimeError {
    #[error(
        "cannot apply binary operator of type: `{operator:?}` between values of type `{}` and `{}`",
        lhs,
        rhs
    )]
    InvalidBinaryOperation {
        /// The debug-symbol of the binary operator.
        operator: BinaryOp,
        /// The value on the left hand side of the expression.
        lhs: Type,
        /// The value on the right hand side of the expression.
        rhs: Type,
    },

    #[error("cannot apply unary operator of type: `{operator:?}` on a value of type `{operand}`")]
    InvalidUnaryOperation {
        /// The debug-symbol of the binary operator.
        operator: UnaryOp,
        /// The value being operated on.
        operand: Type,
    },

    #[error("attempted to divide by zero")]
    DivisionByZero,
}
