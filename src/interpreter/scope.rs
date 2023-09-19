use std::collections::HashMap;

use super::data::Value;

/// A struct that represents a lexical scope.
/// It contains a map of variables and their values,
/// and flags to indicate whether or not to return,
/// break, or continue.
pub struct Scope {
    /// The variables in context.
    pub variables: HashMap<String, Value>,
}

impl Scope {
    /// Creates a new scope.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}
