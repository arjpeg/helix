use thiserror::Error;

use crate::{
    interner::Symbol,
    parser::ast::{BinaryOp, UnaryOp},
    source::Spanned,
    vm::{r#type::Type, value::Value},
};

/// A type alias for the result of an operation that occurred during runtime.
pub type Result<T, E = Spanned<RuntimeError>> = std::result::Result<T, E>;

/// An error that occured during the runtime of the VM.
#[derive(Debug, Clone, Error)]
pub enum RuntimeError {
    #[error(
        "cannot apply binary operator `{operator}` between values of type `{}` and `{}`",
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

    #[error("cannot call value of type: `{}`", callee)]
    NotCallable {
        /// The type value attempted to being called.
        callee: Type,
    },

    #[error("function `{name}` expects {expected} parameters, but received {actual} arguments")]
    MismatchedArity {
        /// The name of the function being called.
        name: Symbol,
        /// The expected number of parameters.
        expected: usize,
        /// The number of arguments actually passed in.
        actual: usize,
    },

    #[error("function `{name}` did not expect argument {n} to be of type: `{received}` ")]
    MismatchedType {
        /// The name of the function being called.
        name: Symbol,
        /// The number of the argument that was the wrong type.
        n: Symbol,
        /// The type actually passed.
        received: Type,
    },

    #[error("assertion failed: expression evaluated to non-truthy value, `{0}`")]
    AssertionFailed(Value),

    #[error("cannot index into value of type: `{base}`")]
    InvalidBase {
        /// The type of the base value attempted to be indexed into.
        base: Type,
    },

    #[error("cannot index into base of type: `{base}` with index: `{index}`")]
    InvalidIndex {
        /// The type of the base value attempted to be indexed into.
        base: Type,
        /// The value of the index.
        index: Value,
    },

    #[error("index: `{index}` was out of bounds for base of length: `{length}`")]
    IndexOutOfBounds {
        /// The value of the index.
        index: Value,
        /// The actual length of the container.
        length: usize,
    },
}
