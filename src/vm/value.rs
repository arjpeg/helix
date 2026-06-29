use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

use itertools::Itertools;

use crate::{
    compiler::{chunk::Function, constants::Constant, index::StackIndex},
    interner::Interner,
    vm::{error::RuntimeError, r#type::Type},
};

/// A value living in the helix runtime environment.
#[derive(Debug, Clone, PartialEq)]
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
    String(Rc<str>),

    /// A heterogeneous array list.
    List(Rc<RefCell<Vec<Value>>>),

    /// A helix-defined function.
    Closure(Rc<Closure>),
}

/// The runtime manifestation of a [`Function`], along with the data it may close over.
#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    /// The function this closure executes (may be shared across different closures).
    pub(crate) function: Rc<Function>,
    /// The list of upvalues this closure closes over
    pub(crate) upvalues: Vec<Rc<RefCell<UpvalueCell>>>,
}

/// A cell holding a captured value from an enclosing scope that may have exited.
#[derive(Debug, Clone, PartialEq)]
pub enum UpvalueCell {
    /// An open reference to a live value on the stack.
    Open(StackIndex),
    /// A closed over value, once the original scope for the upvalue has been dropped.
    Closed(Value),
}

impl From<Constant> for Value {
    fn from(c: Constant) -> Self {
        use Constant as C;

        match c {
            C::Unit => Self::Unit,
            C::Integer(i) => Self::Integer(i),
            C::Float(f) => Self::Float(f.0),
            C::Boolean(b) => Self::Boolean(b),
            C::Symbol(s) => Self::String(Rc::from(Interner::resolve(s))),
        }
    }
}

impl Value {
    /// Returns true if this value is considered truthy or not (largely follows C convention).
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Unit => false,
            Self::Integer(n) => *n != 0,
            Self::Float(n) => *n != 0.0,
            Self::Boolean(b) => *b,
            Self::String(s) => !s.is_empty(),
            Self::List(l) => l.borrow().len() > 0,
            Self::Closure(_) => true,
        }
    }

    /// Returns true if `this` is equal to `other`.
    pub fn equals(this: Self, other: Self) -> Result<Self, RuntimeError> {
        Ok(Self::Boolean(this == other))
    }

    /// Attempts to index into `base` value by the given `index`.
    pub fn index(base: Self, index: Self) -> Result<Self, RuntimeError> {
        let index = match index {
            Self::Integer(i) if i >= 0 => i as usize,

            _ => {
                return Err(RuntimeError::InvalidIndex {
                    base: Type::from(base),
                    index,
                });
            }
        };

        Ok(match base {
            Self::String(string) => {
                if index > string.len() {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: Self::Integer(index as _),
                        length: string.len(),
                    });
                }

                let char = string.chars().nth(index).unwrap().to_string();

                Self::String(Rc::from(char))
            }

            Self::List(list) => {
                let list = list.borrow();

                if index > list.len() {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: Self::Integer(index as _),
                        length: list.len(),
                    });
                }

                list[index].clone()
            }

            _ => {
                return Err(RuntimeError::InvalidBase {
                    base: Type::from(base),
                });
            }
        })
    }

    /// Attempts to assign `value` by indexing into `base` value by the given `index`.
    pub fn index_mut(base: Self, index: Self, value: Self) -> Result<(), RuntimeError> {
        let index = match index {
            Self::Integer(i) if i >= 0 => i as usize,

            _ => {
                return Err(RuntimeError::InvalidIndex {
                    base: Type::from(base),
                    index,
                });
            }
        };

        Ok(match base {
            Self::List(list) => {
                let mut list = list.borrow_mut();

                if index > list.len() {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: Self::Integer(index as _),
                        length: list.len(),
                    });
                }

                list[index] = value;
            }

            _ => {
                return Err(RuntimeError::InvalidBase {
                    base: Type::from(base),
                });
            }
        })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(fl) => write!(f, "{fl}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::String(s) => write!(f, "{s}"),
            Self::List(l) => write!(f, "[{}]", l.borrow().iter().format(", ")),
            Self::Closure(c) => write!(
                f,
                "<fn `{}`>",
                c.function.name.unwrap_or(Interner::intern("(anonymous)"))
            ),
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
    (String(a), String(b)) => String(Rc::from(format!("{a}{b}"))),
    (List(a), List(b)) => List(Rc::from(RefCell::new(a.borrow().iter().chain(b.borrow().iter()).cloned().collect()))),
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

binary_op!(less_than: LessThan, {
    (Float(a), Float(b)) => Boolean(a < b),
    (Integer(a), Integer(b)) => Boolean(a < b),
    (String(a), String(b)) => Boolean(a < b)
});

binary_op!(less_than_equals: LessThanEquals, {
    (Float(a), Float(b)) => Boolean(a <= b),
    (Integer(a), Integer(b)) => Boolean(a <= b),
    (String(a), String(b)) => Boolean(a <= b)
});

unary_op!(negate: Minus, {
    Float(a) => Float(-a),
    Integer(a) => Integer(-a)
});

unary_op!(not: Bang, {
    Integer(a) => Integer(!a),
    Boolean(a) => Boolean(!a)
});
