use std::{cell::RefCell, cmp::Ordering, fmt, rc::Rc};

use crate::{
    interpreter::{Environment, error::RuntimeError},
    parser::ast::{BinaryOp, Expression, UnaryOp},
    source::Spanned,
};

/// A helix value in the living runtime.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// A signed, 64-bit integer.
    Integer(i64),
    /// A boolean.
    Boolean(bool),
    /// An immutable string.
    String(String),
    /// A function defined from helix.
    Function {
        /// The name assigned to this function, or None if it is anonymous.
        name: Option<&'static str>,
        /// The parameters this function accepts.
        parameters: Vec<Spanned<&'static str>>,
        /// The code to call when calling this function.
        body: Spanned<Expression>,
        /// The environment this function captures during creation.
        enclosing: Rc<RefCell<Environment>>,
    },
    /// The unit type, ().
    Unit,
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
            Value::String(_) => "string",
            Value::Function { .. } => "fn",
            Value::Unit => "unit",
        }
    }

    /// Returns if this type is considered truthy.
    ///
    /// The behavior across types is as follows:
    /// * Value::Boolean(b) => returns b
    /// * Value::Integer(n) => returns false if n == 0, true else
    /// * Value::String(s) => returns false if len(s) == 0, true else
    /// * Value::Function() => returns true
    /// * Value::Unit => returns false
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Function { .. } => true,
            Value::Unit => false,
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
            pub fn $name(lhs: Self, rhs: Self) -> Result<Self, super::RuntimeError> {
                #[allow(unused)]
                use Value::*;
                #[allow(unused)]
                use BinaryOp::*;

                #[allow(unreachable_code)]
                match (lhs, rhs) {
                    $( $pattern $(if $guard)? => Ok($body), )*
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
    (Integer(a), Integer(b)) => Integer(a + b),
    (String(a), String(b)) => String(format!("{a}{b}"))
});

binary_op!(subtract: Minus, {
    (Integer(a), Integer(b)) => Integer(a - b)
});

binary_op!(multiply: Star, {
    (Integer(a), Integer(b)) => Integer(a * b),

    (String(_), Integer(n)) if n < 0 => return Err(RuntimeError::NegativeStringRepeat),
    (String(a), Integer(n)) => String(a.repeat(n.try_into().unwrap()))
});

binary_op!(divide: Slash, {
    (Integer(_), Integer(0)) => return Err(RuntimeError::DivideByZero),
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
            (Self::String(a), Self::String(b)) => Some(a.cmp(b)),
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

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{i}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Function {
                name: Some(symbol), ..
            } => write!(f, "fn {symbol} {{ .. }}"),
            Self::Function { .. } => write!(f, "fn {{ .. }}"),
            Self::Unit => write!(f, "()"),
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
    fn str(s: &str) -> Value {
        Value::String(s.to_string())
    }
    fn unit() -> Value {
        Value::Unit
    }

    #[test]
    fn test_integer_arithmetic() {
        assert_eq!(Value::add(int(2), int(3)), Ok(int(5)));
        assert_eq!(Value::subtract(int(5), int(3)), Ok(int(2)));
        assert_eq!(Value::multiply(int(4), int(3)), Ok(int(12)));
        assert_eq!(Value::divide(int(10), int(2)), Ok(int(5)));
    }

    #[test]
    fn test_integer_arithmetic_negatives() {
        assert_eq!(Value::subtract(int(0), int(5)), Ok(int(-5)));
        assert_eq!(Value::multiply(int(-2), int(3)), Ok(int(-6)));
        assert_eq!(Value::divide(int(-10), int(2)), Ok(int(-5)));
    }

    #[test]
    fn test_divide_by_zero() {
        assert_eq!(
            Value::divide(int(1), int(0)),
            Err(RuntimeError::DivideByZero)
        );
    }

    #[test]
    fn test_neg() {
        assert_eq!(Value::neg(int(5)), Ok(int(-5)));
        assert_eq!(Value::neg(int(0)), Ok(int(0)));
        assert_eq!(Value::neg(int(-3)), Ok(int(3)));
    }

    #[test]
    fn test_bitwise_not_integer() {
        assert_eq!(Value::not(int(0)), Ok(int(-1)));
        assert_eq!(Value::not(int(-1)), Ok(int(0)));
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(Value::add(str("foo"), str("bar")), Ok(str("foobar")));
        assert_eq!(Value::add(str(""), str("hello")), Ok(str("hello")));
    }

    #[test]
    fn test_string_repeat() {
        assert_eq!(Value::multiply(str("ab"), int(3)), Ok(str("ababab")));
        assert_eq!(Value::multiply(str("ab"), int(0)), Ok(str("")));
    }

    #[test]
    fn test_string_repeat_negative() {
        assert_eq!(
            Value::multiply(str("ab"), int(-1)),
            Err(RuntimeError::NegativeStringRepeat)
        );
    }

    #[test]
    fn test_int_times_string_fails() {
        assert!(Value::multiply(int(3), str("ab")).is_err());
    }

    #[test]
    fn test_and() {
        assert_eq!(Value::and(bool(true), bool(true)), Ok(bool(true)));
        assert_eq!(Value::and(bool(true), bool(false)), Ok(bool(false)));
        assert_eq!(Value::and(bool(false), bool(false)), Ok(bool(false)));
    }

    #[test]
    fn test_or() {
        assert_eq!(Value::or(bool(false), bool(false)), Ok(bool(false)));
        assert_eq!(Value::or(bool(true), bool(false)), Ok(bool(true)));
    }

    #[test]
    fn test_not_boolean() {
        assert_eq!(Value::not(bool(true)), Ok(bool(false)));
        assert_eq!(Value::not(bool(false)), Ok(bool(true)));
    }

    #[test]
    fn test_integer_ordering() {
        assert_eq!(Value::less_than(int(2), int(3)), Ok(bool(true)));
        assert_eq!(Value::less_than(int(3), int(3)), Ok(bool(false)));
        assert_eq!(Value::greater_than(int(4), int(3)), Ok(bool(true)));
        assert_eq!(Value::greater_than(int(3), int(3)), Ok(bool(false)));
        assert_eq!(Value::less_than_equals(int(3), int(3)), Ok(bool(true)));
        assert_eq!(Value::greater_than_equals(int(3), int(3)), Ok(bool(true)));
    }

    #[test]
    fn test_equality() {
        assert_eq!(Value::equals(int(3), int(3)), Ok(bool(true)));
        assert_eq!(Value::equals(int(3), int(4)), Ok(bool(false)));
        assert_eq!(Value::equals(bool(true), bool(true)), Ok(bool(true)));
        assert_eq!(Value::equals(str("x"), str("x")), Ok(bool(true)));
        assert_eq!(Value::equals(unit(), unit()), Ok(bool(true)));
    }

    #[test]
    fn test_cross_type_equality() {
        assert_eq!(Value::equals(str("1"), int(1)), Ok(bool(false)));
        assert_eq!(Value::equals(int(0), bool(false)), Ok(bool(false)));
        assert_eq!(Value::equals(unit(), int(0)), Ok(bool(false)));
    }

    #[test]
    fn test_arithmetic_type_errors() {
        assert!(Value::add(int(1), bool(true)).is_err());
        assert!(Value::subtract(str("a"), str("b")).is_err());
        assert!(Value::divide(str("a"), int(2)).is_err());
        assert!(Value::add(unit(), unit()).is_err());
    }

    #[test]
    fn test_ordering_type_errors() {
        assert!(Value::less_than(bool(true), bool(false)).is_err());
        assert!(Value::less_than(str("a"), str("b")).is_err());
        assert!(Value::less_than(unit(), unit()).is_err());
    }

    #[test]
    fn test_logic_type_errors() {
        assert!(Value::and(int(1), bool(true)).is_err());
        assert!(Value::or(int(0), bool(false)).is_err());
        assert!(Value::not(unit()).is_err());
        assert!(Value::not(str("x")).is_err());
    }

    #[test]
    fn test_unary_type_errors() {
        assert!(Value::neg(bool(true)).is_err());
        assert!(Value::pos(bool(true)).is_err());
    }

    #[test]
    fn test_is_truthy() {
        assert!(int(1).is_truthy());
        assert!(!int(0).is_truthy());
        assert!(bool(true).is_truthy());
        assert!(!bool(false).is_truthy());
        assert!(str("x").is_truthy());
        assert!(!str("").is_truthy());
        assert!(!unit().is_truthy());
    }

    #[test]
    fn test_display() {
        assert_eq!(int(42).to_string(), "42");
        assert_eq!(int(-1).to_string(), "-1");
        assert_eq!(bool(true).to_string(), "true");
        assert_eq!(bool(false).to_string(), "false");
        assert_eq!(str("hello").to_string(), "hello");
        assert_eq!(unit().to_string(), "()");
    }
}
