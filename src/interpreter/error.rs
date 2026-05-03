use thiserror::Error;

use crate::{
    interpreter::value::Value,
    parser::ast::{BinaryOp, UnaryOp},
};

/// Control-flow signals that propagate up the call stack like errors,
/// but are caught and handled by specific AST nodes rather than surfaced to the user.
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Break,
    Return(Value),
}

/// Either a real error or a control-flow signal in flight.
#[derive(Debug, Clone, PartialEq)]
pub enum Interrupt {
    Error(RuntimeError),
    Signal(Signal),
}

/// An error that occured while running a program.
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

    #[error("cannot call value of type: `{}`", callee.type_name())]
    NotCallable {
        /// The value attempted to being called.
        callee: Value,
    },

    #[error("function `{name}` expects {expected} parameters, but received {actual} arguments")]
    MismatchedArity {
        /// The name of the function being called.
        name: &'static str,
        /// The expected number of parameters.
        expected: usize,
        /// The number of arguments actually passed in.
        actual: usize,
    },

    #[error("attempted to divide by zero")]
    DivideByZero,

    #[error("attempted to repeat a string a negative number of times")]
    NegativeStringRepeat,

    #[error("variable binding `{symbol}` does not exist")]
    UnboundBinding {
        /// The symbol of the binding.
        symbol: &'static str,
    },

    #[error("assertion failed: expression evaluated to non-truthy value, `{0}`")]
    AssertionFailed(Value),

    #[error("attempted to `break` outside a loop")]
    Break,

    #[error("attempted to `return` outside a function context")]
    Return,
}

impl From<RuntimeError> for Interrupt {
    fn from(value: RuntimeError) -> Self {
        Self::Error(value)
    }
}

impl From<Signal> for Interrupt {
    fn from(value: Signal) -> Self {
        Self::Signal(value)
    }
}

impl From<Signal> for RuntimeError {
    fn from(value: Signal) -> Self {
        match value {
            Signal::Break => Self::Break,
            Signal::Return(_) => Self::Return,
        }
    }
}
