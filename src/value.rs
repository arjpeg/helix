use std::fmt::Display;

use crate::{error::Result, token::Span};

macro_rules! impl_binary_operator {
    (
        $( ($name:ident, $operator:ident, {
            $( ($lhs:pat, $rhs:pat) => $body:expr),*
        }) ),*

    ) => {
        $(
            impl Value {
                pub fn $name(&self, other: &Value) -> $crate::error::Result<Value> {
                    use $crate::value::ValueKind::*;
                    use $crate::token::BinaryOperator::*;

                    let span = Span::new(self.span.start..other.span.end, self.span.source);

                    let kind = match (&self.kind, &other.kind) {
                        $( ($lhs, $rhs) => {
                            $body
                        })*
                        _ => return Err($crate::error::Error {
                            span,
                            kind: $crate::error::RuntimeError::InvalidBinaryOperation {
                                lhs: self.kind.clone(),
                                rhs: other.kind.clone(),
                                operator: $operator
                            }.into()
                        }),
                    };

                    Ok($crate::value::Value {
                        kind,
                        span
                    })
                }
            }
        )*
    };
}

macro_rules! impl_unary_operator {
    (
        $( ($name:ident, $operator:ident, {
            $( $operand:pat => $body:expr),*
        }) ),*
    ) => {
        impl Value {
            $(
                pub fn $name(&self) -> $crate::error::Result<Value> {
                    use $crate::value::ValueKind::*;
                    use $crate::token::UnaryOperator::*;

                    let span = self.span.clone();

                    let kind = match &self.kind {
                        $( $operand  => {
                            $body
                        })*

                        _ => return Err($crate::error::Error {
                            span,
                            kind: $crate::error::RuntimeError::InvalidUnaryOperation {
                                operand: self.kind.clone(),
                                operator: $operator
                            }.into()
                        }),
                    };

                    Ok($crate::value::Value {
                        kind,
                        span
                    })
                }
            )*
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub kind: ValueKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    /// A floating point number.
    Float(f64),
    /// An integer.
    Integer(i64),
    /// A boolean.
    Boolean(bool),
}

impl Value {
    /// Create a new value.
    pub fn new(kind: ValueKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl_binary_operator! {
    (add, Plus, {
        (Float(a), Float(b)) => Float(a + b),
        (Integer(a), Integer(b)) => Integer(a + b)
    }),

    (subtract, Minus, {
        (Float(a), Float(b)) => Float(a - b),
        (Integer(a), Integer(b)) => Integer(a - b)
    }),

    (multiply, Multiply, {
        (Float(a), Float(b)) => Float(a * b),
        (Integer(a), Integer(b)) => Integer(a * b)
    }),

    (divide, Divide, {
        (Float(a), Float(b)) => Float(a / b),
        (Integer(a), Integer(b)) => Integer(a / b)
    }),

    (less_than, LessThan, {
        (Float(a), Float(b)) => Boolean(a < b),
        (Integer(a), Integer(b)) => Boolean(a < b)
    }),

    (less_than_or_equal, LessThanEquals, {
        (Float(a), Float(b)) => Boolean(a <= b),
        (Integer(a), Integer(b)) => Boolean(a <= b)
    }),

    (greater_than, GreaterThan, {
        (Float(a), Float(b)) => Boolean(a > b),
        (Integer(a), Integer(b)) => Boolean(a > b)
    }),

    (greater_than_or_equal, GreaterThanEquals, {
        (Float(a), Float(b)) => Boolean(a >= b),
        (Integer(a), Integer(b)) => Boolean(a >= b)
    }),

    (equal, Equals, {
        (Float(a), Float(b)) => Boolean(a == b),
        (Integer(a), Integer(b)) => Boolean(a == b),
        (Boolean(a), Boolean(b)) => Boolean(a == b)
    })
}

impl Value {
    pub fn not_equal(&self, other: &Value) -> Result<Value> {
        self.equal(other)?.not()
    }
}

impl_unary_operator! {
    (not, Not, {
        Boolean(b) => Boolean(!b)
    }),

    (negate, Minus, {
        Float(f) => Float(-f),
        Integer(i) => Integer(-i)
    })
}

impl ValueKind {
    /// Returns the canonical name of this value kind.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Float(_) => "float",
            Self::Integer(_) => "integer",
            Self::Boolean(_) => "boolean",
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.kind))
    }
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Float(f) => f.to_string(),
            Self::Integer(i) => i.to_string(),
            Self::Boolean(b) => b.to_string(),
        })
    }
}
