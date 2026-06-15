use crate::{
    compiler::instruction::{Instruction, OpCode, disassemble_instruction},
    source::Span,
};

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

            Instruction::GetGlobal { name_index: index } => {
                self.code.push(index);
            }

            Instruction::GetLocal { stack_index: index } => {
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

            Instruction::GetGlobal { name_index } => {
                println!(
                    "GET_GLOBAL ({name_index} : {:?})",
                    chunk.constants[name_index as usize]
                );
            }

            Instruction::SetGlobal { name_index } => {
                println!(
                    "SET_GLOBAL ({name_index} : {:?})",
                    chunk.constants[name_index as usize]
                );
            }

            Instruction::GetLocal { stack_index } => {
                println!("GET_LOCAL ({stack_index})",)
            }

            Instruction::SetLocal { stack_index } => {
                println!("SET_LOCAL ({stack_index})",)
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
