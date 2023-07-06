use crate::lexer::token::OperatorKind;

use super::error::InterpreterError::*;
use super::InterpreterResult;

/// A generic data type.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A number.
    Number(f64),

    /// A boolean.
    Boolean(bool),

    /// Nothing.
    Null,
}

macro_rules! impl_binary_op {
    ($name:ident $operator:ident { $(($lhs:pat, $rhs:pat) => $body:expr),* $(,)? }) => {
        pub fn $name(&self, other: Value) -> InterpreterResult<Value> {
            #[allow(unused_imports)]
            use Value::*;

            match (self, other.clone()) {
                $(
                    ($lhs, $rhs) => $body,
                )*

                _ => Err(InvalidBinaryExpression {
                    operator: OperatorKind::$operator,
                    lhs: self.clone(),
                    rhs: other,
                }),
            }
        }
    };
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Number(number) => *number != 0.0,
            Value::Boolean(boolean) => *boolean,
            Value::Null => false,
        }
    }

    impl_binary_op!(add Plus {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs + rhs)),
    });

    impl_binary_op!(subtract Minus {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs - rhs)),
    });

    impl_binary_op!(multiply Star {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs * rhs)),
    });

    impl_binary_op!(divide Slash {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs / rhs)),
    });

    impl_binary_op!(power Power {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs.powf(rhs))),
    });
}
