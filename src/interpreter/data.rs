use crate::lexer::span::Span;
use crate::lexer::token::OperatorKind;

use super::error::InterpreterError::*;
use super::InterpreterResult;

/// A generic data type.
#[derive(Debug, Clone)]
pub struct Value {
    /// The kind of the data.
    pub kind: ValueKind,

    /// The span of the data.
    pub span: Span,
}

/// The kind of the data.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ValueKind {
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
            use ValueKind::*;

            match (self.clone().kind, other.clone().kind) {
                $(
                    // If $body returns Ok(ValueKind), then wrap it in a Value.
                    // Otherwise, return the error.
                    ($lhs, $rhs) => $body.map(|kind| Value {
                        kind,
                        span: (self.span.start..other.span.end).into(),
                    }),
                )*

                _ => Err(InvalidBinaryExpression {
                    operator: OperatorKind::$operator,
                    lhs: self.clone(),
                    rhs: other.clone(),
                    span: (self.span.start..other.span.end).into(),
                }),
            }
        }
    };
}

impl Value {
    /// Returns whether or not the value is truthy.
    #[allow(dead_code)]
    pub fn is_truthy(&self) -> bool {
        use ValueKind::*;

        match self.kind {
            Number(number) => number != 0.0,
            Boolean(boolean) => boolean,
            Null => false,
        }
    }

    impl_binary_op!(add Plus {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs + rhs)),
    });

    impl_binary_op!(subtract Minus {
        // (Number(lhs), Number(rhs)) => Ok(Number(lhs - rhs)),
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
