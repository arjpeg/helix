use std::{collections::HashMap, ops::Index};

use ordered_float::OrderedFloat;

use crate::interner::{Interner, Symbol};

/// A constant referred to within a [`Chunk`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    /// The unit type, also known as `()`.
    Unit,

    /// A 64-bit, signed integer.
    Integer(i64),
    /// A 64-bit, floating point number.
    Float(OrderedFloat<f64>),

    /// A logical boolean.
    Boolean(bool),

    /// A symbol in the source code.
    Symbol(Symbol),
}

/// A deduplicated pool of constants stored per chunk.
#[derive(Debug, Clone, Default)]
pub struct ConstantPool {
    /// A list of the constants currently stored, with a maximum of 256 constants.
    list: Vec<Constant>,
    /// A mapping from already added constant: index into `list`.
    index_map: HashMap<Constant, u8>,
}

impl ConstantPool {
    /// Creates a new, empty [`ConstantPool`].
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
            index_map: HashMap::new(),
        }
    }

    /// Adds a constant to the pool if it wasn't already added, returning the index to it.
    pub fn insert(&mut self, c: Constant) -> u8 {
        if let Some(&index) = self.index_map.get(&c) {
            return index;
        }

        let len = self.list.len();

        assert!(
            len <= u8::MAX as usize,
            "constant pool doesn't have space for any more constants"
        );

        self.list.push(c);
        self.index_map.insert(c, len as u8);

        len as u8
    }

    /// Retrieves a constant from the pool by index.
    pub fn get(&self, index: usize) -> Constant {
        self.list[index]
    }
}

impl Index<u8> for ConstantPool {
    type Output = Constant;

    fn index(&self, index: u8) -> &Self::Output {
        &self.list[index as usize]
    }
}

impl From<i64> for Constant {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Constant {
    fn from(value: f64) -> Self {
        Self::Float(OrderedFloat(value))
    }
}

impl From<bool> for Constant {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<&'static str> for Constant {
    fn from(value: &'static str) -> Self {
        Self::Symbol(Interner::intern(value))
    }
}
