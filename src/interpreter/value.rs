use crate::lexer::token::OpKind;

/// A helix value in the living runtime.
#[derive(Debug, Clone)]
pub enum Value {
    /// An integer.
    Integer(u64),
}

impl Value {
    /// Performs a binary operation between two values.
    pub fn binary_operation(lhs: Self, operator: OpKind, rhs: Self) -> Result<Self, ()> {
        match operator {
            OpKind::Plus => Self::add(lhs, rhs),
            // TODO: impl rest
            _ => Err(()),
        }
    }
}

/// Creates an implementation of a binary operation reducer between two `Values`.
macro_rules! binary_op {
    (
        $name:ident,
        {
            $( $pattern:pat => $body:expr ),* $(,)?
        }
    ) => {
        impl Value {
            pub fn $name(lhs: Self, rhs: Self) -> Result<Self, ()> {
                #[allow(unused)]
                use Value::*;

                #[allow(unreachable_patterns)]
                match (lhs, rhs) {
                    $( $pattern => Ok($body), )*
                    _ => Err(()),
                }
            }
        }
    };
}

binary_op!(add, {
    (Integer(a), Integer(b)) => Integer(a + b)
});
