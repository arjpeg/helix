use std::fmt::Display;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::compiler::index::{
    ConstantIndex, FunctionIndex, InstructionPointer, LocalIndex, StackIndex, UpvalueIndex,
};

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
    /// Pops `n` values from under the top of the stack, leaving the top in place.
    PopUnder {
        /// The number of values to pop.
        n: u8,
    },

    /// Closes all open upvalues whose absolute stack pointer >= `from`.
    CloseAbove {
        /// The base stack address to close over from.
        from: StackIndex,
    },

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
    LoadConstant(ConstantIndex),
    /// Allocates a new closure object by loading a template from the function pool.
    MakeClosure(FunctionIndex),

    /// Invokes the execution of a function-like value.
    Call {
        /// The number of arguments passed in.
        arguments: u8,
    },

    /// Creates a new list of length `length`, consuming `length` values from the top of the stack to
    /// populate it.
    MakeList { length: u8 },
    /// Indexes into the second from top value on the stack by the top value on the stack, popping
    /// both and placing the result back at the top of the stack.
    GetIndex,
    /// Indexes into the third from top value on the stack by the second value on the stack,
    /// and sets its value to the top value on the stack, popping all three values.
    SetIndex,

    /// Declares a new global variable with the value as the popped value on the top of the stack.
    DefineGlobal(ConstantIndex),
    /// Reads the value of a previously declared global variable, placing it at the top of the
    /// stack.
    GetGlobal(ConstantIndex),
    /// Updates the value of a global variable by popping the top of the stack.
    SetGlobal(ConstantIndex),

    /// Reads the value of a captured upvalue within a closure.
    GetUpvalue(UpvalueIndex),
    /// Updates the value of a captured upvalue within a closure.
    SetUpvalue(UpvalueIndex),

    /// Retrives a local variable allocated at a given stack location.
    GetLocal(LocalIndex),
    /// Updates the value of a local variable by popping the top of the stack.
    SetLocal(LocalIndex),

    /// Prints the popped value at the top of the stack.
    Print,

    /// Asserts that the popped value at the top of the stack is truthy.
    Assert,

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
    /// See [Instruction::PopUnder].
    PopUnder,
    /// See [Instruction::CloseAbove].
    CloseAbove,
    /// See [Instruction::Jump].
    Jump,
    /// See [Instruction::JumpIfTrue].
    JumpIfTrue,
    /// See [Instruction::JumpIfFalse].
    JumpIfFalse,
    /// See [Instruction::LoadConstant].
    LoadConstant,
    /// See [Instruction::MakeClosure].
    MakeClosure,
    /// See [Instruction::Call].
    Call,
    /// See [Instruction::MakeList].
    MakeList,
    /// See [Instruction::GetIndex].
    GetIndex,
    /// See [Instruction::SetIndex].
    SetIndex,
    /// See [Instruction::DefineGlobal].
    DefineGlobal,
    /// See [Instruction::GetGlobal].
    GetGlobal,
    /// See [Instruction::SetGlobal].
    SetGlobal,
    /// See [Instruction::GetUpvalue].
    GetUpvalue,
    /// See [Instruction::SetUpvalue].
    SetUpvalue,
    /// See [Instruction::GetLocal].
    GetLocal,
    /// See [Instruction::SetLocal].
    SetLocal,
    /// See [Instruction::Print].
    Print,
    /// See [Instruction::Assert].
    Assert,
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
            Instruction::PopUnder { n } => buf.push(n),
            Instruction::CloseAbove { from } => buf.extend(from.0.to_ne_bytes()),

            Instruction::LoadConstant(ConstantIndex(index)) => buf.push(index),
            Instruction::MakeClosure(FunctionIndex(index)) => buf.push(index),

            Instruction::Call { arguments } => buf.push(arguments),

            Instruction::MakeList { length } => buf.push(length),

            Instruction::Jump { offset } => buf.extend(offset.to_ne_bytes()),
            Instruction::JumpIfTrue { offset } => buf.extend(offset.to_ne_bytes()),
            Instruction::JumpIfFalse { offset } => buf.extend(offset.to_ne_bytes()),

            Instruction::DefineGlobal(ConstantIndex(index)) => buf.push(index),
            Instruction::GetGlobal(ConstantIndex(index)) => buf.push(index),
            Instruction::SetGlobal(ConstantIndex(index)) => buf.push(index),

            Instruction::GetUpvalue(UpvalueIndex(index)) => buf.push(index),
            Instruction::SetUpvalue(UpvalueIndex(index)) => buf.push(index),

            Instruction::GetLocal(LocalIndex(index)) => buf.push(index),
            Instruction::SetLocal(LocalIndex(index)) => buf.push(index),

            _ => {}
        }
    }

    /// Decodes an instruction starting from `start` of `buf`, returning the decoded instruction and
    /// the index to the start of the next instruction (if any).
    pub fn decode(
        buf: &[u8],
        InstructionPointer(start): InstructionPointer,
    ) -> (Self, InstructionPointer) {
        let opcode = OpCode::try_from_primitive(buf[start])
            .expect("dissassembler started on invalid instruction");

        let (instruction, next) = match opcode {
            // simple one byte instructions
            OpCode::Return => (Instruction::Return, start + 1),
            OpCode::Duplicate => (Instruction::Duplicate, start + 1),
            OpCode::Pop => (Instruction::Pop, start + 1),
            OpCode::GetIndex => (Instruction::GetIndex, start + 1),
            OpCode::SetIndex => (Instruction::SetIndex, start + 1),
            OpCode::Print => (Instruction::Print, start + 1),
            OpCode::Assert => (Instruction::Assert, start + 1),
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
            OpCode::PopUnder => (Instruction::PopUnder { n: buf[start + 1] }, start + 2),
            OpCode::CloseAbove => (
                Instruction::CloseAbove {
                    from: StackIndex(u16::from_ne_bytes([buf[start + 1], buf[start + 2]])),
                },
                start + 3,
            ),
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
                Instruction::LoadConstant(ConstantIndex(buf[start + 1])),
                start + 2,
            ),
            OpCode::MakeClosure => (
                Instruction::MakeClosure(FunctionIndex(buf[start + 1])),
                start + 2,
            ),
            OpCode::Call => (
                Instruction::Call {
                    arguments: buf[start + 1],
                },
                start + 2,
            ),
            OpCode::MakeList => (
                Instruction::MakeList {
                    length: buf[start + 1],
                },
                start + 2,
            ),
            OpCode::DefineGlobal => (
                Instruction::DefineGlobal(ConstantIndex(buf[start + 1])),
                start + 2,
            ),
            OpCode::GetGlobal => (
                Instruction::GetGlobal(ConstantIndex(buf[start + 1])),
                start + 2,
            ),
            OpCode::SetGlobal => (
                Instruction::SetGlobal(ConstantIndex(buf[start + 1])),
                start + 2,
            ),
            OpCode::GetUpvalue => (
                Instruction::GetUpvalue(UpvalueIndex(buf[start + 1])),
                start + 2,
            ),
            OpCode::SetUpvalue => (
                Instruction::SetUpvalue(UpvalueIndex(buf[start + 1])),
                start + 2,
            ),
            OpCode::GetLocal => (Instruction::GetLocal(LocalIndex(buf[start + 1])), start + 2),
            OpCode::SetLocal => (Instruction::SetLocal(LocalIndex(buf[start + 1])), start + 2),
        };

        (instruction, InstructionPointer(next))
    }
}

impl From<&Instruction> for OpCode {
    fn from(value: &Instruction) -> Self {
        match value {
            Instruction::Return => Self::Return,
            Instruction::Duplicate => Self::Duplicate,
            Instruction::Pop => Self::Pop,
            Instruction::PopUnder { .. } => Self::PopUnder,
            Instruction::CloseAbove { .. } => Self::CloseAbove,
            Instruction::Jump { .. } => Self::Jump,
            Instruction::JumpIfTrue { .. } => Self::JumpIfTrue,
            Instruction::JumpIfFalse { .. } => Self::JumpIfFalse,
            Instruction::LoadConstant { .. } => Self::LoadConstant,
            Instruction::MakeClosure { .. } => Self::MakeClosure,
            Instruction::Call { .. } => Self::Call,
            Instruction::MakeList { .. } => Self::MakeList,
            Instruction::GetIndex { .. } => Self::GetIndex,
            Instruction::SetIndex { .. } => Self::SetIndex,
            Instruction::DefineGlobal { .. } => Self::DefineGlobal,
            Instruction::GetGlobal { .. } => Self::GetGlobal,
            Instruction::SetGlobal { .. } => Self::SetGlobal,
            Instruction::GetUpvalue { .. } => Self::GetUpvalue,
            Instruction::SetUpvalue { .. } => Self::SetUpvalue,
            Instruction::GetLocal { .. } => Self::GetLocal,
            Instruction::SetLocal { .. } => Self::SetLocal,
            Instruction::Print => Self::Print,
            Instruction::Assert => Self::Assert,
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
            OpCode::PopUnder => "POP_UNDER",
            OpCode::CloseAbove => "CLOSE_ABOVE",
            OpCode::Jump => "JUMP",
            OpCode::JumpIfTrue => "JUMP_IF_TRUE",
            OpCode::JumpIfFalse => "JUMP_IF_FALSE",
            OpCode::LoadConstant => "LOAD_CONSTANT",
            OpCode::MakeClosure => "MAKE_CLOSURE",
            OpCode::Call => "CALL",
            OpCode::MakeList => "MAKE_LIST",
            OpCode::GetIndex => "GET_INDEX",
            OpCode::SetIndex => "SET_INDEX",
            OpCode::DefineGlobal => "DEFINE_GLOBAL",
            OpCode::GetGlobal => "GET_GLOBAL",
            OpCode::SetGlobal => "SET_GLOBAL",
            OpCode::GetUpvalue => "GET_UPVALUE",
            OpCode::SetUpvalue => "SET_UPVALUE",
            OpCode::GetLocal => "GET_LOCAL",
            OpCode::SetLocal => "SET_LOCAL",
            OpCode::Print => "PRINT",
            OpCode::Assert => "ASSERT",
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
        write!(f, "{}\t", OpCode::from(self))?;

        match self {
            Instruction::PopUnder { n } => write!(f, "N:{n}")?,
            Instruction::CloseAbove { from } => write!(f, "{from}")?,
            Instruction::Jump { offset } => write!(f, "O:{offset}")?,
            Instruction::JumpIfTrue { offset } => write!(f, "O:{offset}")?,
            Instruction::JumpIfFalse { offset } => write!(f, "O:{offset}")?,
            Instruction::LoadConstant(index) => write!(f, "{index}")?,
            Instruction::MakeClosure(index) => write!(f, "{index}")?,
            Instruction::Call { arguments } => write!(f, "A:{arguments}")?,
            Instruction::MakeList { length } => write!(f, "(length: {length})")?,
            Instruction::DefineGlobal(index) => write!(f, "{index}")?,
            Instruction::GetGlobal(index) => write!(f, "{index}")?,
            Instruction::SetGlobal(index) => write!(f, "{index}")?,
            Instruction::GetUpvalue(index) => write!(f, "{index}")?,
            Instruction::SetUpvalue(index) => write!(f, "{index}")?,
            Instruction::GetLocal(index) => write!(f, "{index}")?,
            Instruction::SetLocal(index) => write!(f, "{index}")?,
            _ => {}
        };

        Ok(())
    }
}
