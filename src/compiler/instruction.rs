use std::fmt::Display;

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// All the different instructions performed by the VM.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Return from the current function context, or stop program execution from the global
    /// scope.
    Return,

    /// Duplicates the topmost value on the stack.
    Duplicate,
    /// Pops the topmost value from the stack, discarding the result.
    Pop,

    /// Unconditionally jumps by the given offset..
    Jump {
        /// The offset from the instruction pointer to jump by.
        offset: i16,
    },
    /// Jumps by the given offset if the popped value at the top of the stack is truthy.
    JumpIfTrue {
        /// The offset from the instruction pointer to jump by.
        offset: i16,
    },
    /// Jumps by the given offset if the popped value at the top of the stack is falsey.
    JumpIfFalse {
        /// The offset from the instruction pointer to jump by.
        offset: i16,
    },

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

    /// Prints the popped value at the top of the stack.
    Print,

    /// Computes `stack.pop() + stack.pop()`, appending the result back to the stack.
    Add,
    /// Computes `stack.pop() - stack.pop()`, appending the result back to the stack.
    Subtract,
    /// Computes `stack.pop() * stack.pop()`, appending the result back to the stack.
    Multiply,
    /// Computes `stack.pop() / stack.pop()`, appending the result back to the stack.
    Divide,

    /// Computes `stack.pop() == stack.pop()`, appending the result back to the stack.
    Equals,

    /// Computes `stack.pop() < stack.pop()`, appending the result back to the stack.
    ///
    /// Can be used to compute greater than, by swapping the order arguments are placed on the
    /// stack.
    LessThan,
    /// Computes `stack.pop() <= stack.pop()`, appending the result back to the stack.
    ///
    /// Can be used to compute greater than equals, by swapping the order arguments are placed on the
    /// stack.
    LessThanEquals,

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
    /// See [Instruction::Duplicate].
    Duplicate,
    /// See [Instruction::Pop].
    Pop,
    /// See [Instruction::Jump].
    Jump,
    /// See [Instruction::JumpIfTrue].
    JumpIfTrue,
    /// See [Instruction::JumpIfFalse].
    JumpIfFalse,
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
    /// See [Instruction::Print].
    Print,
    /// See [Instruction::Add].
    Add,
    /// See [Instruction::Subtract]
    Subtract,
    /// See [Instruction::Multiply].
    Multiply,
    /// See [Instruction::Divide].
    Divide,
    /// See [Instruction::Equals].
    Equals,
    /// See [Instruction::LessThan].
    LessThan,
    /// See [Instruction::LessThanEquals].
    LessThanEquals,
    /// See [Instruction::Negate].
    Negate,
    /// See [Instruction::Not].
    Not,
}

impl Instruction {
    /// Encodes this instruction as a sequence of bytes, appending it to `buf`.
    pub fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(OpCode::from(self) as u8);

        match *self {
            Instruction::LoadConstant { index } => buf.push(index),

            Instruction::Jump { offset } => buf.extend(offset.to_ne_bytes()),
            Instruction::JumpIfTrue { offset } => buf.extend(offset.to_ne_bytes()),
            Instruction::JumpIfFalse { offset } => buf.extend(offset.to_ne_bytes()),

            Instruction::DefineGlobal { index } => buf.push(index),
            Instruction::GetGlobal { name_index } => buf.push(name_index),
            Instruction::SetGlobal { name_index } => buf.push(name_index),

            Instruction::GetLocal { stack_index } => buf.push(stack_index),
            Instruction::SetLocal { stack_index } => buf.push(stack_index),

            _ => {}
        }
    }

    /// Decodes an instruction starting from `start` of `buf`, returning the decoded instruction and
    /// the index to the start of the next instruction (if any).
    pub fn decode(buf: &[u8], start: usize) -> (Self, usize) {
        let opcode = OpCode::try_from_primitive(buf[start])
            .expect("dissassembler started on invalid instruction");

        match opcode {
            // simple one byte instructions
            OpCode::Return => (Instruction::Return, start + 1),
            OpCode::Duplicate => (Instruction::Duplicate, start + 1),
            OpCode::Pop => (Instruction::Pop, start + 1),
            OpCode::Print => (Instruction::Print, start + 1),
            OpCode::Add => (Instruction::Add, start + 1),
            OpCode::Subtract => (Instruction::Subtract, start + 1),
            OpCode::Multiply => (Instruction::Multiply, start + 1),
            OpCode::Divide => (Instruction::Divide, start + 1),
            OpCode::Equals => (Instruction::Equals, start + 1),
            OpCode::LessThan => (Instruction::LessThan, start + 1),
            OpCode::LessThanEquals => (Instruction::LessThanEquals, start + 1),
            OpCode::Negate => (Instruction::Negate, start + 1),
            OpCode::Not => (Instruction::Not, start + 1),

            // multi byte instructions
            OpCode::Jump => (
                Instruction::Jump {
                    offset: i16::from_ne_bytes([buf[start + 1], buf[start + 2]]),
                },
                start + 3,
            ),
            OpCode::JumpIfTrue => (
                Instruction::JumpIfTrue {
                    offset: i16::from_ne_bytes([buf[start + 1], buf[start + 2]]),
                },
                start + 3,
            ),
            OpCode::JumpIfFalse => (
                Instruction::JumpIfFalse {
                    offset: i16::from_ne_bytes([buf[start + 1], buf[start + 2]]),
                },
                start + 3,
            ),
            OpCode::LoadConstant => (
                Instruction::LoadConstant {
                    index: buf[start + 1],
                },
                start + 2,
            ),
            OpCode::DefineGlobal => (
                Instruction::DefineGlobal {
                    index: buf[start + 1],
                },
                start + 2,
            ),
            OpCode::GetGlobal => (
                Instruction::GetGlobal {
                    name_index: buf[start + 1],
                },
                start + 2,
            ),
            OpCode::SetGlobal => (
                Instruction::SetGlobal {
                    name_index: buf[start + 1],
                },
                start + 2,
            ),
            OpCode::GetLocal => (
                Instruction::GetLocal {
                    stack_index: buf[start + 1],
                },
                start + 2,
            ),
            OpCode::SetLocal => (
                Instruction::SetLocal {
                    stack_index: buf[start + 1],
                },
                start + 2,
            ),
        }
    }
}

impl From<&Instruction> for OpCode {
    fn from(value: &Instruction) -> Self {
        match value {
            Instruction::Return => Self::Return,
            Instruction::Duplicate => Self::Duplicate,
            Instruction::Pop => Self::Pop,
            Instruction::Jump { .. } => Self::Jump,
            Instruction::JumpIfTrue { .. } => Self::JumpIfTrue,
            Instruction::JumpIfFalse { .. } => Self::JumpIfFalse,
            Instruction::LoadConstant { .. } => Self::LoadConstant,
            Instruction::DefineGlobal { .. } => Self::DefineGlobal,
            Instruction::GetGlobal { .. } => Self::GetGlobal,
            Instruction::SetGlobal { .. } => Self::SetGlobal,
            Instruction::GetLocal { .. } => Self::GetLocal,
            Instruction::SetLocal { .. } => Self::SetLocal,
            Instruction::Print => Self::Print,
            Instruction::Add => Self::Add,
            Instruction::Subtract => Self::Subtract,
            Instruction::Multiply => Self::Multiply,
            Instruction::Divide => Self::Divide,
            Instruction::Equals => Self::Equals,
            Instruction::LessThan => Self::LessThan,
            Instruction::LessThanEquals => Self::LessThanEquals,
            Instruction::Negate => Self::Negate,
            Instruction::Not => Self::Not,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            OpCode::Return => "RETURN",
            OpCode::Duplicate => "DUPLICATE",
            OpCode::Pop => "POP",
            OpCode::Jump => "JUMP",
            OpCode::JumpIfTrue => "JUMP_IF_TRUE",
            OpCode::JumpIfFalse => "JUMP_IF_FALSE",
            OpCode::LoadConstant => "LOAD_CONSTANT",
            OpCode::DefineGlobal => "DEFINE_GLOBAL",
            OpCode::GetGlobal => "GET_GLOBAL",
            OpCode::SetGlobal => "SET_GLOBAL",
            OpCode::GetLocal => "GET_LOCAL",
            OpCode::SetLocal => "SET_LOCAL",
            OpCode::Print => "PRINT",
            OpCode::Add => "ADD",
            OpCode::Subtract => "SUBTRACT",
            OpCode::Multiply => "MULTIPLY",
            OpCode::Divide => "DIVIDE",
            OpCode::Equals => "EQUALS",
            OpCode::LessThan => "LESS_THAN",
            OpCode::LessThanEquals => "LESS_THAN_EQUALS",
            OpCode::Negate => "NEGATE",
            OpCode::Not => "NOT",
        })
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", OpCode::from(self))?;

        match self {
            Instruction::Jump { offset } => write!(f, " O:{offset:}")?,
            Instruction::JumpIfTrue { offset } => write!(f, " O:{offset:}")?,
            Instruction::JumpIfFalse { offset } => write!(f, " O:{offset:}")?,
            Instruction::LoadConstant { index } => write!(f, " C:{index}")?,
            Instruction::DefineGlobal { index } => write!(f, " C:{index}")?,
            Instruction::GetGlobal { name_index } => write!(f, " C:{name_index}")?,
            Instruction::SetGlobal { name_index } => write!(f, " C:{name_index}")?,
            Instruction::GetLocal { stack_index } => write!(f, " S:{stack_index}")?,
            Instruction::SetLocal { stack_index } => write!(f, " S:{stack_index}")?,
            _ => {}
        };

        Ok(())
    }
}
