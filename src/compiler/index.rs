//! The different indexing spaces used withing the compiler and runtime.

use std::fmt::Display;

/// An index to a [`Constant`] held within a [`Chunk`]'s [`ConstantPool`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstantIndex(pub u8);

/// An index to a static function held within [`Chunk::functions`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionIndex(pub u8);

/// An index to an instruction held within [`Chunk::code`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstructionPointer(pub usize);

/// An offset from a base slot for a local variable during function invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalIndex(pub u8);

/// An absolute location on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StackIndex(pub u16);

/// An index to an upvalue in the current frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpvalueIndex(pub u8);

impl InstructionPointer {
    /// Offsets the instruction pointer by the given (potentially negative) offset.
    pub fn offset(self, by: i16) -> Self {
        Self(self.0.saturating_add_signed(by as isize))
    }

    /// Returns a pointer to the next byte.
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Returns a pointer to the previous byte.
    pub fn previous(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

impl Display for ConstantIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(constant: #{})", self.0)
    }
}

impl Display for FunctionIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(function: #{})", self.0)
    }
}

impl Display for InstructionPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:4x}", self.0)
    }
}

impl Display for LocalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(local: #{})", self.0)
    }
}

impl Display for StackIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(stack: #{})", self.0)
    }
}

impl Display for UpvalueIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(upvalue: #{})", self.0)
    }
}
