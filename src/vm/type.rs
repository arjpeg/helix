use crate::vm::value::Value;

/// The different *types* [`Value`]s can take on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    /// The unit type, also known as `()`.
    Unit,

    /// A 64-bit signed integer.
    Integer,
    /// A 64-bit floating point number.
    Float,
    /// A logical boolean.
    Boolean,
    /// A utf-8 encoded immutable string.
    String,
    /// A helix-defined function.
    Closure,
}

impl From<Value> for Type {
    fn from(value: Value) -> Self {
        match value {
            Value::Unit => Self::Unit,
            Value::Integer(_) => Self::Integer,
            Value::Float(_) => Self::Float,
            Value::Boolean(_) => Self::Boolean,
            Value::String(_) => Self::String,
            Value::Closure(_) => Self::Closure,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Unit => "unit",
                Type::Integer => "integer",
                Type::Float => "float",
                Type::Boolean => "boolean",
                Type::String => "string",
                Type::Closure => "function",
            }
        )
    }
}
