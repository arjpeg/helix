use std::collections::HashMap;

use super::data::Value;

/// A struct that represents a lexical scope.
/// It contains a map of variables and their values,
/// and flags to indicate whether or not to return,
/// break, or continue.
pub struct Scope {
    /// The scope's name (used for function/module names).
    pub name: Option<String>,

    /// The variables in context.
    pub variables: HashMap<String, Value>,

    /// Whether or not to return.
    pub return_value: Option<Value>,

    /// Whether or not to break.
    pub should_break: bool,

    /// Whether or not to continue.
    pub should_continue: bool,
}

impl Scope {
    /// Creates a new scope.
    pub fn new() -> Self {
        Self {
            name: None,
            variables: HashMap::new(),
            return_value: None,
            should_break: false,
            should_continue: false,
        }
    }
}
