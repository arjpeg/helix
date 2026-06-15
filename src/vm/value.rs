use crate::{compiler::chunk::Constant, interner::Interner, vm::error::RuntimeError};

/// A value living in the helix runtime environment.
#[derive(Debug, Clone)]
pub enum Value {
    /// The unit type, also known as `()`.
    Unit,

    /// A 64-bit signed integer.
    Integer(i64),
    /// A 64-bit floating point number.
    Float(f64),
    /// A logical boolean.
    Boolean(bool),
    /// A utf-8 encoded immutable string.
    String(Box<str>),
}

impl From<Constant> for Value {
    fn from(c: Constant) -> Self {
        use Constant as C;

        match c {
            C::Unit => Self::Unit,
            C::Integer(i) => Self::Integer(i),
            C::Float(f) => Self::Float(f),
            C::Boolean(b) => Self::Boolean(b),
            C::Symbol(s) => Self::String(Box::from(Interner::resolve(s))),
        }
    }
}

/// Creates an implementation of a binary operation reducer between two [`Value`]s.
macro_rules! binary_op {
    (
        $name:ident: $operator:ident,
        {
            $( $pattern:pat $(if $guard:expr)? => $body:expr ),* $(,)?
        }
    ) => {
        impl Value {

            pub fn $name(lhs: Self, rhs: Self) -> Result<Self, crate::vm::error::RuntimeError> {
                #[allow(unused)]
                use Value::*;
                #[allow(unused)]
                use crate::parser::ast::BinaryOp::*;

                #[allow(unreachable_code)]
                match (lhs, rhs) {
                    $( $pattern $(if $guard)? => Ok($body), )*
                    (lhs, rhs) => Err(crate::vm::error::RuntimeError::InvalidBinaryOperation {
                        operator: $operator,
                        lhs: lhs.into(),
                        rhs: rhs.into(),
                    }),
                }
            }
        }
    };
}

/// Creates an implementation of a unary operation reducer acting on a [`Value`].
macro_rules! unary_op {
    (
        $name:ident: $operator:ident,
        {
            $( $pattern:pat => $body:expr ),* $(,)?
        }
    ) => {
        impl Value {
            pub fn $name(operand: Self) -> Result<Self, crate::vm::error::RuntimeError> {
                #[allow(unused)]
                use Value::*;
                #[allow(unused)]
                use crate::parser::ast::UnaryOp::*;

                #[allow(unreachable_patterns)]
                match (operand) {
                    $( $pattern => Ok($body), )*
                    operand => Err(crate::vm::error::RuntimeError::InvalidUnaryOperation {
                        operator: $operator,
                        operand: operand.into()
                    }),
                }
            }
        }
    };
}

binary_op!(add: Plus, {
    (Float(a), Float(b)) => Float(a + b),
    (Integer(a), Integer(b)) => Integer(a + b),
});

binary_op!(subtract: Minus, {
    (Float(a), Float(b)) => Float(a - b),
    (Integer(a), Integer(b)) => Integer(a - b),
});

binary_op!(multiply: Star, {
    (Float(a), Float(b)) => Float(a * b),
    (Integer(a), Integer(b)) => Integer(a * b),
});

binary_op!(divide: Slash, {
    (Float(_), Float(0.0)) => return Err(RuntimeError::DivisionByZero),
    (Integer(_), Integer(0)) => return Err(RuntimeError::DivisionByZero),

    (Float(a), Float(b)) => Float(a / b),
    (Integer(a), Integer(b)) => Integer(a / b),
});

unary_op!(negate: Minus, {
    Float(a) => Float(-a),
    Integer(a) => Integer(-a)
});

unary_op!(not: Bang, {
    Integer(a) => Integer(!a),
    Boolean(a) => Boolean(!a)
});
