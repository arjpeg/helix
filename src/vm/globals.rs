use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::{interner::Symbol, vm::value::Value};

/// Manages synchronization of global variables across compilation and program execution.
#[derive(Clone)]
pub struct Globals {
    /// The globals defined at compile time.
    pub known: HashSet<Symbol>,

    /// The runtime values of the globals actually used.
    pub runtime: HashMap<Symbol, Value>,
}

impl Globals {
    /// Creates a new [`Globals`] value with no initial global variables.
    pub fn new() -> Self {
        Self {
            known: HashSet::new(),
            runtime: HashMap::new(),
        }
    }

    /// Creates a snapshot of the current state of global variables.
    pub fn snapshot(&self) -> Self {
        self.clone()
    }
}

impl Debug for Globals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Globals")
            .field("rt", &self.runtime)
            .finish()
    }
}
