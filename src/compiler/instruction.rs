use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::compiler::chunk::Chunk;

/// All the different instructions performed by the VM.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Return from the current function context, or stop program execution from the global
    /// scope.
    Return,
    /// Pops the topmost value from the stack, discarding the result.
    Pop,

    /// Loads a constant from the constant pool.
    LoadConstant {
        /// The index of constant to load.
        index: u8,
    },

    /// Declares a new global variable with the value as the top value on the stack, which then
    /// gets popped.
    DefineGlobal {
        /// The index of the constant containing the name of the variable.
        index: u8,
    },

    /// Reads the value of a previously declared global variable, placing it at the top of the
    /// stack.
    GetGlobal {
        /// The index of the constant containing the name of the variable.
        name_index: u8,
    },
    /// Updates the value of a global variable by copying the top of the stack.
    SetGlobal {
        /// The index of the constant containing the name of the variable.
        name_index: u8,
    },

    /// Retrives a local variable allocated at a given stack location.
    GetLocal {
        /// The index on the stack to load.
        stack_index: u8,
    },

    /// Updates the value of a local variable by copying the top of the stack.
    SetLocal {
        /// The index on the stack to update.
        stack_index: u8,
    },

    /// Adds the top two values on the stack, popping them and then appending the
    /// result back onto the stack.
    Add,
    /// Subtracts the top two values on the stack, popping them and then appending the
    /// result back onto the stack.
    Subtract,
    /// Subtracts the top two values on the stack, popping them and then appending the
    /// result back onto the stack.
    Multiply,
    /// Subtracts the top two values on the stack, popping them and then appending the
    /// result back onto the stack.
    Divide,

    /// Negates the top value on the stack in place.
    Negate,
    /// Applies the not operation on the top value on the stack in place.
    Not,
}

/// The operation code associated with each [`Instruction`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum OpCode {
    /// See [Instruction::Return].
    Return = 0,
    /// See [Instruction::Pop].
    Pop,
    /// See [Instruction::LoadConstant].
    LoadConstant,
    /// See [Instruction::DefineGlobal].
    DefineGlobal,
    /// See [Instruction::GetGlobal].
    GetGlobal,
    /// See [Instruction::SetGlobal].
    SetGlobal,
    /// See [Instruction::GetLocal].
    GetLocal,
    /// See [Instruction::SetLocal].
    SetLocal,
    /// See [Instruction::Add].
    Add,
    /// See [Instruction::Subtract]
    Subtract,
    /// See [Instruction::Multiply].
    Multiply,
    /// See [Instruction::Divide].
    Divide,
    /// See [Instruction::Negate].
    Negate,
    /// See [Instruction::Not].
    Not,
}

impl From<&Instruction> for OpCode {
    fn from(value: &Instruction) -> Self {
        match value {
            Instruction::Return => Self::Return,
            Instruction::Pop => Self::Pop,
            Instruction::LoadConstant { .. } => Self::LoadConstant,
            Instruction::DefineGlobal { .. } => Self::DefineGlobal,
            Instruction::GetGlobal { .. } => Self::GetGlobal,
            Instruction::SetGlobal { .. } => Self::SetGlobal,
            Instruction::GetLocal { .. } => Self::GetLocal,
            Instruction::SetLocal { .. } => Self::SetLocal,
            Instruction::Add => Self::Add,
            Instruction::Subtract => Self::Subtract,
            Instruction::Multiply => Self::Multiply,
            Instruction::Divide => Self::Divide,
            Instruction::Negate => Self::Negate,
            Instruction::Not => Self::Not,
        }
    }
}

pub(crate) fn disassemble_instruction(chunk: &Chunk, offset: usize) -> (Instruction, usize) {
    let opcode = OpCode::try_from_primitive(chunk.code[offset])
        .expect("dissassembler started on invalid instruction");

    match opcode {
        // simple one byte instructions
        OpCode::Return => (Instruction::Return, offset + 1),
        OpCode::Pop => (Instruction::Pop, offset + 1),
        OpCode::Add => (Instruction::Add, offset + 1),
        OpCode::Subtract => (Instruction::Subtract, offset + 1),
        OpCode::Multiply => (Instruction::Multiply, offset + 1),
        OpCode::Divide => (Instruction::Divide, offset + 1),
        OpCode::Negate => (Instruction::Negate, offset + 1),
        OpCode::Not => (Instruction::Not, offset + 1),

        // multi byte instructions
        OpCode::LoadConstant => (
            Instruction::LoadConstant {
                index: chunk.code[offset + 1],
            },
            offset + 2,
        ),
        OpCode::DefineGlobal => (
            Instruction::DefineGlobal {
                index: chunk.code[offset + 1],
            },
            offset + 2,
        ),
        OpCode::GetGlobal => (
            Instruction::GetGlobal {
                name_index: chunk.code[offset + 1],
            },
            offset + 2,
        ),
        OpCode::SetGlobal => (
            Instruction::GetGlobal {
                name_index: chunk.code[offset + 1],
            },
            offset + 2,
        ),
        OpCode::GetLocal => (
            Instruction::GetLocal {
                stack_index: chunk.code[offset + 1],
            },
            offset + 2,
        ),
        OpCode::SetLocal => (
            Instruction::GetLocal {
                stack_index: chunk.code[offset + 1],
            },
            offset + 2,
        ),
    }
}
