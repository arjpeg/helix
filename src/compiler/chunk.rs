use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::source::Span;

/// A sequence of bytecode generated from an Abstract Syntax Tree.
#[derive(Clone)]
pub struct Chunk {
    /// The compiled set of flattened [`Instruction`]s.
    pub(crate) code: Vec<u8>,

    /// The pool of [`Constant`]s loaded into this chunk.
    pub(crate) constants: Vec<Constant>,

    /// The [`Span`]s associated with each instruction in the `code`, with one span per
    /// instruction.
    ///
    /// Stored in the format (instruction start in `code`, span).
    pub(crate) spans: Vec<(usize, Span)>,

    /// The debug name of this chunk.
    pub(crate) name: Option<&'static str>,
}

/// A constant referred to within a [`Chunk`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constant {
    /// The unit type, also known as `()`.
    Unit,

    /// A 64-bit, signed integer.
    Integer(i64),
    /// A 64-bit, floating point number.
    Float(f64),

    /// A logical boolean.
    Boolean(bool),

    /// A symbol in the source code.
    /// TODO: add string interning
    Symbol(&'static str),
}

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
        index: u8,
    },

    /// Retrives a local variable allocated at a given stack location.
    GetLocal {
        /// The index on the stack to load.
        index: u8,
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
    /// See [Instruction::GetLocal].
    GetLocal,
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

impl Chunk {
    /// Creates a new, empty [`Chunk`].
    pub fn new(name: Option<&'static str>) -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
            name,
        }
    }

    /// Appends one [`Constant`] to this chunk, returning the index of the constant within the
    /// `constant_pool`.
    pub fn emit_constant(&mut self, constant: Constant) -> u8 {
        // TODO: add constant deduping
        assert!(
            self.constants.len() <= u8::MAX as usize,
            "constant pool doesn't have space for any more constants"
        );

        let position = self.constants.len();
        self.constants.push(constant);
        position as u8
    }

    /// Appends one [`Instruction`] to this chunk, returning the index of the start of the
    /// emitted instruction.
    pub fn emit_instruction(&mut self, instruction: Instruction, span: Span) -> usize {
        let start = self.code.len();
        self.spans.push((start, span));

        // push the opcode
        self.code.push(OpCode::from(&instruction) as u8);

        // push additional operations for longer instructions
        match instruction {
            Instruction::LoadConstant { index } => {
                self.code.push(index);
            }

            Instruction::DefineGlobal { index } => {
                self.code.push(index);
            }

            Instruction::GetGlobal { index } => {
                self.code.push(index);
            }

            Instruction::GetLocal { index } => {
                self.code.push(index);
            }

            _ => {}
        }

        start
    }

    /// Returns the left span corresponding to the given code offset.
    pub fn span_at(&self, offset: usize) -> Span {
        let i = self.spans.partition_point(|&(start, _)| start <= offset);

        self.spans[i - 1].1
    }
}

/// Disassemble a [`Chunk`] into a format suitable for debugging.
pub fn disassemble(chunk: &Chunk) {
    println!("== {} ==", chunk.name.unwrap_or("<anonymous>"));

    let mut offset = 0;

    while offset < chunk.code.len() {
        let (instruction, next) = disassemble_instruction(chunk, offset);
        let span = chunk.span_at(offset);

        print!("{:0>4} ({}..{})  ", offset, span.start, span.end);

        match instruction {
            Instruction::Return => println!("RETURN"),

            Instruction::Pop => println!("POP"),

            Instruction::LoadConstant { index } => {
                println!(
                    "LOAD_CONSTANT ({index} : {:?})",
                    chunk.constants[index as usize]
                )
            }

            Instruction::DefineGlobal { index } => {
                println!(
                    "DEFINE_GLOBAL ({index} : {:?})",
                    chunk.constants[index as usize]
                );
            }

            Instruction::GetGlobal { index } => {
                println!(
                    "GET_GLOBAL ({index} : {:?})",
                    chunk.constants[index as usize]
                );
            }

            Instruction::GetLocal { index } => {
                println!("GET_LOCAL ({index})",)
            }

            Instruction::Add => println!("ADD"),

            Instruction::Subtract => println!("SUBTRACT"),

            Instruction::Multiply => println!("MULTIPLY"),

            Instruction::Divide => println!("DIVIDE"),

            Instruction::Negate => println!("NEGATE"),

            Instruction::Not => println!("NOT"),
        }

        offset = next;
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
                index: chunk.code[offset + 1],
            },
            offset + 2,
        ),
        OpCode::GetLocal => (
            Instruction::GetLocal {
                index: chunk.code[offset + 1],
            },
            offset + 2,
        ),
    }
}

impl From<&Instruction> for OpCode {
    fn from(value: &Instruction) -> Self {
        match value {
            Instruction::Return => Self::Return,
            Instruction::Pop => Self::Pop,
            Instruction::LoadConstant { .. } => Self::LoadConstant,
            Instruction::DefineGlobal { .. } => Self::DefineGlobal,
            Instruction::GetGlobal { .. } => Self::GetGlobal,
            Instruction::GetLocal { .. } => Self::GetLocal,
            Instruction::Add => Self::Add,
            Instruction::Subtract => Self::Subtract,
            Instruction::Multiply => Self::Multiply,
            Instruction::Divide => Self::Divide,
            Instruction::Negate => Self::Negate,
            Instruction::Not => Self::Not,
        }
    }
}

impl From<i64> for Constant {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for Constant {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for Constant {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<&'static str> for Constant {
    fn from(value: &'static str) -> Self {
        Self::Symbol(value)
    }
}
