use crate::{
    compiler::{
        constants::{Constant, ConstantPool},
        instruction::Instruction,
    },
    source::Span,
};

/// A sequence of bytecode generated from an Abstract Syntax Tree.
#[derive(Clone)]
pub struct Chunk {
    /// The compiled set of flattened [`Instruction`]s.
    pub(crate) code: Vec<u8>,

    /// The pool of [`Constant`]s loaded into this chunk.
    pub(crate) constants: ConstantPool,

    /// The [`Span`]s associated with each instruction in the `code`, with one span per
    /// instruction.
    ///
    /// Stored in the format (instruction start in `code`, span).
    pub(crate) spans: Vec<(usize, Span)>,

    /// The debug name of this chunk.
    pub(crate) name: Option<&'static str>,
}

impl Chunk {
    /// Creates a new, empty [`Chunk`].
    pub fn new(name: Option<&'static str>) -> Self {
        Self {
            code: Vec::new(),
            constants: ConstantPool::new(),
            spans: Vec::new(),
            name,
        }
    }

    /// Appends one [`Constant`] to this chunk, returning the index of the constant within the
    /// `constant_pool`.
    pub fn emit_constant(&mut self, constant: Constant) -> u8 {
        self.constants.insert(constant)
    }

    /// Appends one [`Instruction`] to this chunk, returning the index of the start of the
    /// emitted instruction.
    pub fn emit_instruction(&mut self, instruction: Instruction, span: Span) -> usize {
        let start = self.code.len();

        self.spans.push((start, span));
        instruction.encode(&mut self.code);

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

    println!("constants: {:?}", chunk.constants);

    let mut offset = 0;

    while offset < chunk.code.len() {
        let (instruction, next) = Instruction::decode(&chunk.code, offset);
        let span = chunk.span_at(offset);

        println!(
            "{:0>4} ({}..{})  {instruction}",
            offset, span.start, span.end
        );

        offset = next;
    }
}
