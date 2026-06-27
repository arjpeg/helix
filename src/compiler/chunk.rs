use std::rc::Rc;

use crate::{
    compiler::{
        constants::{Constant, ConstantPool},
        instruction::Instruction,
    },
    interner::{Interner, Symbol},
    source::Span,
    vm::value::Function,
};

/// A sequence of bytecode generated from an Abstract Syntax Tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    /// The compiled set of flattened [`Instruction`]s.
    pub(crate) code: Vec<u8>,

    /// The pool of [`Constant`]s loaded into this chunk.
    pub(crate) constants: ConstantPool,
    /// The pool of [`Function`]s compiled within this chunk.
    pub(crate) functions: Vec<Rc<Function>>,

    /// The [`Span`]s associated with each instruction in the `code`, with one span per
    /// instruction.
    ///
    /// Stored in the format (instruction start in `code`, span).
    pub(crate) spans: Vec<(usize, Span)>,

    /// The debug name of this chunk.
    pub(crate) name: Option<Symbol>,
}

impl Chunk {
    /// Creates a new, empty [`Chunk`].
    pub fn new(name: Option<Symbol>) -> Self {
        Self {
            code: Vec::new(),
            constants: ConstantPool::new(),
            functions: Vec::new(),
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

    /// Backpatches the bytecode for an [`Instruction::Jump`] (and variants) starting at `base` to
    /// point at the head of the bytecode.
    ///
    /// If `to` is `None`, the jump is backpatched to the current [`Chunk::position`].
    pub fn backpatch_jump(&mut self, base: usize, to: Option<usize>) {
        // the opcode takes up one byte, starting at base,
        // so we modify the following two bytes

        let to = to.unwrap_or(self.position()) as i16;
        let base = base as i16;

        // 3 here being one byte for opcode + 2 bytes for offset
        let offset = if to > base {
            to - base - 3
        } else {
            to - base - 3
        };

        let [a, b] = offset.to_ne_bytes();
        self.code[base as usize + 1] = a;
        self.code[base as usize + 2] = b;
    }

    /// Returns the length, or current position of the chunk.
    pub fn position(&mut self) -> usize {
        self.code.len()
    }

    /// Returns the left span corresponding to the given code offset.
    pub fn span_at(&self, offset: usize) -> Span {
        let i = self.spans.partition_point(|&(start, _)| start <= offset);

        self.spans[i - 1].1
    }
}

/// Disassemble a [`Chunk`] into a format suitable for debugging.
pub fn disassemble(chunk: &Chunk) {
    println!(
        "== {} ==",
        chunk.name.unwrap_or(Interner::intern("<anonymous>"))
    );

    println!("constants: {:?}", chunk.constants);

    let mut offset = 0;

    while offset < chunk.code.len() {
        let (instruction, next) = Instruction::decode(&chunk.code, offset);
        let span = chunk.span_at(offset);

        println!(
            "{:0>4} ({: >2}..{: <2})  {instruction}",
            offset, span.start, span.end
        );

        offset = next;
    }
}
