use std::fmt;

use crate::interpreter::error::InterpreterError;
use crate::lexer::span::Span;
use crate::lexer::token::OperatorKind;
use crate::parser::ast::AstNode;

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

    /// A string.
    String(String),

    /// A boolean.
    Boolean(bool),

    /// A function.
    Function {
        name: String,
        parameters: Vec<String>,
        body: AstNode,
    },

    /// Nothing.
    Null,
}

macro_rules! impl_binary_op {
    ($name:ident, $operator:ident, { $(($lhs:pat, $rhs:pat $(, $span:ident)? $(,)?) => $body:expr),* $(,)? }) => {
        pub fn $name(&self, other: &Value) -> InterpreterResult<Value> {
            #[allow(unused_imports)]
            use ValueKind::*;

            let expr_span: Span = (self.span.start..other.span.end).into();

            match (self.clone().kind, other.clone().kind) {
                $(
                    ($lhs, $rhs) => {
                        $(
                            let $span = expr_span.clone();
                        )?

                        $body.map(|kind| Value {
                            kind,
                            span: expr_span.clone(),
                        })
                    },
                )*

                _ => Err(InterpreterError::InvalidBinaryExpression {
                    operator: OperatorKind::$operator,
                    lhs: self.clone(),
                    rhs: other.clone(),
                    span: (self.span.start..other.span.end).into(),
                }),
            }
        }
    };
}

macro_rules! impl_unary_op {
    ($name:ident, $operator:ident, { $(($lhs:pat $(, $span:ident)? $(,)?) => $body:expr),* $(,)? }) => {
        pub fn $name(&self) -> InterpreterResult<Value> {
            #[allow(unused_imports)]
            use ValueKind::*;

            let expr_span: Span = self.span;

            match (self.clone().kind) {
                $(
                    $lhs => {
                        $(
                            let $span = expr_span.clone();
                        )?

                        $body.map(|kind| Value {
                            kind,
                            span: expr_span.clone(),
                        })
                    },
                )*

                _ => Err(InterpreterError::InvalidUnaryExpression {
                    operator: OperatorKind::$operator,
                    expr: self.clone(),
                    span: (self.span.start-1..self.span.end).into(),
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

        match self.kind.clone() {
            Number(number) => number != 0.0,
            Boolean(boolean) => boolean,
            String(string) => !string.is_empty(),
            Function { .. } => true,
            Null => false,
        }
    }

    // Binary Operations
    impl_binary_op!(add, Plus, {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs + rhs)),
        (String(lhs), String(rhs)) => Ok(String(lhs + &rhs)),
    });

    impl_binary_op!(subtract, Minus, {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs - rhs)),
    });

    impl_binary_op!(multiply, Star, {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs * rhs)),
        (String(lhs), Number(rhs)) => Ok(String(lhs.repeat(rhs as usize)))
    });

    impl_binary_op!(divide, Slash, {
        (Number(lhs), Number(rhs), span) => {
            if rhs == 0.0 {
                Err(InterpreterError::DivisionByZero { span })
            } else {
                Ok(Number(lhs / rhs))
            }
        },
    });

    impl_binary_op!(power, Power, {
        (Number(lhs), Number(rhs)) => Ok(Number(lhs.powf(rhs))),
    });

    impl_unary_op!(negate, Minus, {
        (Number(lhs)) => Ok(Number(-lhs)),
    });

    impl_binary_op!(less_than, LessThan, {
        (Number(lhs), Number(rhs)) => Ok(Boolean(lhs < rhs)),
    });

    impl_binary_op!(less_than_or_equal, LessThanOrEqual, {
        (Number(lhs), Number(rhs)) => Ok(Boolean(lhs <= rhs)),
    });

    impl_binary_op!(greater_than, GreaterThan, {
        (Number(lhs), Number(rhs)) => Ok(Boolean(lhs > rhs)),
    });

    impl_binary_op!(greater_than_or_equal, GreaterThanOrEqual, {
        (Number(lhs), Number(rhs)) => Ok(Boolean(lhs >= rhs)),
    });

    impl_binary_op!(equals, Equals, {
        (Number(lhs), Number(rhs)) => Ok(Boolean(lhs == rhs)),
    });
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValueKind::*;

        match self.clone() {
            Number(number) => write!(f, "{}", number),
            String(string) => write!(f, "{}", string),
            Boolean(boolean) => write!(f, "{}", boolean),
            Function {
                name, parameters, ..
            } => {
                write!(f, "function {name}({})", parameters.join(", "))
            }
            Null => write!(f, "null"),
        }
    }
}
