use thiserror::Error;

use crate::{
    interpreter::value::Value,
    parser::ast::{BinaryOp, UnaryOp},
};

/// An error that occured while running the program.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RuntimeError {
    #[error(
        "cannot apply binary operator of type: `{operator:?}` between values of type `{}` and `{}`",
        lhs.type_name(),
        rhs.type_name()
    )]
    InvalidBinaryOperation {
        /// The debug-symbol of the binary operator.
        operator: BinaryOp,
        /// The value on the left hand side of the expression.
        lhs: Value,
        /// The value on the right hand side of the expression.
        rhs: Value,
    },

    #[error(
        "cannot apply unary operator of type: `{operator:?}` on a value of type `{}`",
        operand.type_name()
    )]
    InvalidUnaryOperation {
        /// The debug-symbol of the binary operator.
        operator: UnaryOp,
        /// The value being operated on.
        operand: Value,
    },

    #[error("attempted to divide by zero")]
    DivideByZero,
}
