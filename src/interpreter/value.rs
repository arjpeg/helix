use std::{cmp::Ordering, fmt};

use crate::{
    interpreter::error::RuntimeError,
    parser::ast::{BinaryOp, UnaryOp},
};

/// A helix value in the living runtime.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// An integer.
    Integer(i64),
    /// A boolean.
    Boolean(bool),
}

impl Value {
    /// Performs a binary operation between two [`Value`]s.
    pub fn binary_operation(
        lhs: Self,
        operator: BinaryOp,
        rhs: Self,
    ) -> Result<Self, RuntimeError> {
        // TODO: implement short-circuiting?
        match operator {
            BinaryOp::Plus => Self::add(lhs, rhs),
            BinaryOp::Minus => Self::subtract(lhs, rhs),
            BinaryOp::Star => Self::multiply(lhs, rhs),
            BinaryOp::Slash => Self::divide(lhs, rhs),
            BinaryOp::NotEquals => Self::not_equals(lhs, rhs),
            BinaryOp::Equals => Self::equals(lhs, rhs),
            BinaryOp::GreaterThan => Self::greater_than(lhs, rhs),
            BinaryOp::GreaterThanEquals => Self::greater_than_equals(lhs, rhs),
            BinaryOp::LessThan => Self::less_than(lhs, rhs),
            BinaryOp::LessThanEquals => Self::less_than_equals(lhs, rhs),
            BinaryOp::And => Self::and(lhs, rhs),
            BinaryOp::Or => Self::or(lhs, rhs),
        }
    }

    /// Performs a unary operation on a [`Value`].
    pub fn unary_operation(operator: UnaryOp, operand: Self) -> Result<Self, RuntimeError> {
        match operator {
            UnaryOp::Plus => Self::pos(operand),
            UnaryOp::Minus => Self::neg(operand),
            UnaryOp::Bang => Self::not(operand),
        }
    }

    /// Attempts to extract the [`Value`] as an integer, returning the underlying u64.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Attempts to extract the [`Value`] as a boolean.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the canonical type name of this [`Value`].
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Boolean(_) => "boolean",
        }
    }
}

/// Creates an implementation of a binary operation reducer between two [`Value`]s.
macro_rules! binary_op {
    (
        $name:ident: $operator:ident,
        {
            $( $pattern:pat => $body:expr ),* $(,)?
        }
    ) => {
        impl Value {
            pub fn $name(lhs: Self, rhs: Self) -> Result<Self, super::RuntimeError> {
                #[allow(unused)]
                use Value::*;
                #[allow(unused)]
                use BinaryOp::*;

                #[allow(unreachable_patterns)]
                match (lhs, rhs) {
                    $( $pattern => Ok($body), )*
                    (lhs, rhs) => Err(super::RuntimeError::InvalidBinaryOperation {
                        operator: $operator,
                        lhs,
                        rhs,
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
            pub fn $name(operand: Self) -> Result<Self, RuntimeError> {
                #[allow(unused)]
                use Value::*;
                #[allow(unused)]
                use UnaryOp::*;

                #[allow(unreachable_patterns)]
                match (operand) {
                    $( $pattern => Ok($body), )*
                    operand => Err(super::RuntimeError::InvalidUnaryOperation {
                        operator: $operator,
                        operand
                    }),
                }
            }
        }
    };
}

binary_op!(add: Plus, {
    (Integer(a), Integer(b)) => Integer(a + b)
});

binary_op!(subtract: Minus, {
    (Integer(a), Integer(b)) => Integer(a - b)
});

binary_op!(multiply: Star, {
    (Integer(a), Integer(b)) => Integer(a * b)
});

binary_op!(divide: Slash, {
    (Integer(a), Integer(b)) => Integer(a / b)
});

binary_op!(and: And, {
    (Boolean(a), Boolean(b)) => Boolean(a && b)
});

binary_op!(or: Or, {
    (Boolean(a), Boolean(b)) => Boolean(a || b)
});

unary_op!(pos: Plus, {
    Integer(a) => Integer(a)
});

unary_op!(neg: Minus, {
    Integer(a) => Integer(-a)
});

unary_op!(not: Bang, {
    Integer(a) => Integer(!a),
    Boolean(a) => Boolean(!a)
});

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

impl Value {
    pub fn equals(lhs: Self, rhs: Self) -> Result<Self, RuntimeError> {
        Ok(Self::Boolean(lhs == rhs))
    }

    pub fn not_equals(lhs: Self, rhs: Self) -> Result<Self, RuntimeError> {
        Ok(Self::Boolean(lhs != rhs))
    }

    /// Attempts to get the relative ordering between two [`Value`]s, returning a [`RuntimeError`]
    /// if the operator could not be applied.
    fn compare(lhs: Self, rhs: Self, operator: BinaryOp) -> Result<Ordering, RuntimeError> {
        lhs.partial_cmp(&rhs)
            .ok_or(RuntimeError::InvalidBinaryOperation { operator, lhs, rhs })
    }

    pub fn less_than(lhs: Self, rhs: Self) -> Result<Self, RuntimeError> {
        Ok(Self::Boolean(
            Self::compare(lhs, rhs, BinaryOp::LessThan)? == Ordering::Less,
        ))
    }

    pub fn greater_than(lhs: Self, rhs: Self) -> Result<Self, RuntimeError> {
        Ok(Self::Boolean(
            Self::compare(lhs, rhs, BinaryOp::GreaterThan)? == Ordering::Greater,
        ))
    }

    pub fn less_than_equals(lhs: Self, rhs: Self) -> Result<Self, RuntimeError> {
        Ok(Self::Boolean(matches!(
            Self::compare(lhs, rhs, BinaryOp::LessThanEquals)?,
            Ordering::Less | Ordering::Equal
        )))
    }

    pub fn greater_than_equals(lhs: Self, rhs: Self) -> Result<Self, RuntimeError> {
        Ok(Self::Boolean(matches!(
            Self::compare(lhs, rhs, BinaryOp::GreaterThanEquals)?,
            Ordering::Greater | Ordering::Equal
        )))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{i}"),
            Self::Boolean(b) => write!(f, "{b}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(n: i64) -> Value {
        Value::Integer(n)
    }
    fn bool(b: bool) -> Value {
        Value::Boolean(b)
    }

    #[test]
    fn test_add() {
        assert_eq!(Value::add(int(2), int(3)), Ok(int(5)));
    }

    #[test]
    fn test_subtract() {
        assert_eq!(Value::subtract(int(5), int(3)), Ok(int(2)));
    }

    #[test]
    fn test_multiply() {
        assert_eq!(Value::multiply(int(4), int(3)), Ok(int(12)));
    }

    #[test]
    fn test_divide() {
        assert_eq!(Value::divide(int(10), int(2)), Ok(int(5)));
    }

    #[test]
    #[should_panic]
    fn test_divide_by_zero() {
        let _ = Value::divide(int(1), int(0));
    }

    #[test]
    fn test_add_type_mismatch() {
        assert!(Value::add(int(1), bool(true)).is_err());
    }

    #[test]
    fn test_ordering_on_booleans_fails() {
        assert!(Value::less_than(bool(true), bool(false)).is_err());
        assert!(Value::greater_than(bool(true), bool(false)).is_err());
        assert!(Value::less_than_equals(bool(true), bool(false)).is_err());
        assert!(Value::greater_than_equals(bool(true), bool(false)).is_err());
    }

    #[test]
    fn test_equals_integers() {
        assert_eq!(Value::equals(int(3), int(3)), Ok(bool(true)));
        assert_eq!(Value::equals(int(3), int(4)), Ok(bool(false)));
    }

    #[test]
    fn test_equals_booleans() {
        assert_eq!(Value::equals(bool(true), bool(true)), Ok(bool(true)));
        assert_eq!(Value::equals(bool(true), bool(false)), Ok(bool(false)));
    }

    #[test]
    fn test_not_equals() {
        assert_eq!(Value::not_equals(int(3), int(4)), Ok(bool(true)));
        assert_eq!(Value::not_equals(int(3), int(3)), Ok(bool(false)));
        assert_eq!(Value::not_equals(bool(true), bool(false)), Ok(bool(true)));
    }

    #[test]
    fn test_less_than() {
        assert_eq!(Value::less_than(int(2), int(3)), Ok(bool(true)));
        assert_eq!(Value::less_than(int(3), int(3)), Ok(bool(false)));
        assert_eq!(Value::less_than(int(4), int(3)), Ok(bool(false)));
    }

    #[test]
    fn test_greater_than() {
        assert_eq!(Value::greater_than(int(4), int(3)), Ok(bool(true)));
        assert_eq!(Value::greater_than(int(3), int(3)), Ok(bool(false)));
        assert_eq!(Value::greater_than(int(2), int(3)), Ok(bool(false)));
    }

    #[test]
    fn test_less_than_equals() {
        assert_eq!(Value::less_than_equals(int(2), int(3)), Ok(bool(true)));
        assert_eq!(Value::less_than_equals(int(3), int(3)), Ok(bool(true)));
        assert_eq!(Value::less_than_equals(int(4), int(3)), Ok(bool(false)));
    }

    #[test]
    fn test_greater_than_equals() {
        assert_eq!(Value::greater_than_equals(int(4), int(3)), Ok(bool(true)));
        assert_eq!(Value::greater_than_equals(int(3), int(3)), Ok(bool(true)));
        assert_eq!(Value::greater_than_equals(int(2), int(3)), Ok(bool(false)));
    }

    #[test]
    fn test_and() {
        assert_eq!(Value::and(bool(true), bool(true)), Ok(bool(true)));
        assert_eq!(Value::and(bool(true), bool(false)), Ok(bool(false)));
        assert_eq!(Value::and(bool(false), bool(false)), Ok(bool(false)));
    }

    #[test]
    fn test_and_type_mismatch() {
        assert!(Value::and(int(1), bool(true)).is_err());
    }

    #[test]
    fn test_or() {
        assert_eq!(Value::or(bool(false), bool(false)), Ok(bool(false)));
        assert_eq!(Value::or(bool(true), bool(false)), Ok(bool(true)));
        assert_eq!(Value::or(bool(true), bool(true)), Ok(bool(true)));
    }

    #[test]
    fn test_or_type_mismatch() {
        assert!(Value::or(int(1), bool(true)).is_err());
    }

    #[test]
    fn test_pos() {
        assert_eq!(Value::pos(int(5)), Ok(int(5)));
        assert_eq!(Value::pos(int(-3)), Ok(int(-3)));
    }

    #[test]
    fn test_pos_on_boolean_fails() {
        assert!(Value::pos(bool(true)).is_err());
    }

    #[test]
    fn test_neg() {
        assert_eq!(Value::neg(int(5)), Ok(int(-5)));
        assert_eq!(Value::neg(int(-3)), Ok(int(3)));
        assert_eq!(Value::neg(int(0)), Ok(int(0)));
    }

    #[test]
    fn test_neg_on_boolean_fails() {
        assert!(Value::neg(bool(true)).is_err());
    }

    #[test]
    fn test_not_boolean() {
        assert_eq!(Value::not(bool(true)), Ok(bool(false)));
        assert_eq!(Value::not(bool(false)), Ok(bool(true)));
    }

    #[test]
    fn test_not_integer() {
        assert_eq!(Value::not(int(0)), Ok(int(-1)));
        assert_eq!(Value::not(int(-1)), Ok(int(0)));
    }

    #[test]
    fn test_binary_operation_dispatch() {
        assert_eq!(
            Value::binary_operation(int(2), BinaryOp::Plus, int(3)),
            Ok(int(5))
        );
        assert_eq!(
            Value::binary_operation(int(5), BinaryOp::Minus, int(3)),
            Ok(int(2))
        );
        assert_eq!(
            Value::binary_operation(int(3), BinaryOp::Equals, int(3)),
            Ok(bool(true))
        );
        assert_eq!(
            Value::binary_operation(int(3), BinaryOp::NotEquals, int(4)),
            Ok(bool(true))
        );
        assert_eq!(
            Value::binary_operation(int(2), BinaryOp::LessThan, int(3)),
            Ok(bool(true))
        );
        assert_eq!(
            Value::binary_operation(int(4), BinaryOp::GreaterThan, int(3)),
            Ok(bool(true))
        );
        assert_eq!(
            Value::binary_operation(bool(true), BinaryOp::And, bool(false)),
            Ok(bool(false))
        );
        assert_eq!(
            Value::binary_operation(bool(false), BinaryOp::Or, bool(true)),
            Ok(bool(true))
        );
    }

    #[test]
    fn test_unary_operation_dispatch() {
        assert_eq!(Value::unary_operation(UnaryOp::Minus, int(5)), Ok(int(-5)));
        assert_eq!(
            Value::unary_operation(UnaryOp::Bang, bool(true)),
            Ok(bool(false))
        );
        assert_eq!(Value::unary_operation(UnaryOp::Plus, int(3)), Ok(int(3)));
    }

    #[test]
    fn test_from() {
        assert_eq!(Value::from(42i64), int(42));
        assert_eq!(Value::from(true), bool(true));
    }

    #[test]
    fn test_as_integer() {
        assert_eq!(int(5).as_integer(), Some(5));
        assert_eq!(bool(true).as_integer(), None);
    }

    #[test]
    fn test_as_boolean() {
        assert_eq!(bool(false).as_boolean(), Some(false));
        assert_eq!(int(1).as_boolean(), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(int(42).to_string(), "42");
        assert_eq!(bool(true).to_string(), "true");
        assert_eq!(bool(false).to_string(), "false");
    }

    #[test]
    fn test_type_name() {
        assert_eq!(int(1).type_name(), "integer");
        assert_eq!(bool(true).type_name(), "boolean");
    }
}
