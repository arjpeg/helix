use std::{ops::Index, rc::Rc};

use crate::{
    compiler::{
        Upvalue,
        constants::{Constant, ConstantPool},
        index::{ConstantIndex, FunctionIndex, InstructionPointer},
        instruction::Instruction,
    },
    interner::{Interner, Symbol},
    source::Span,
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

/// The metadata representing a complete function defined in helix.
#[derive(Clone, PartialEq)]
pub struct Function {
    /// The number of parameters this function accepts.
    pub(crate) arity: u8,
    /// The descriptors of the [`Upvalue`]s this function closes over.
    pub(crate) upvalues: Vec<Upvalue>,
    /// The bytecode [`Chunk`] to execute when this function is invoked.
    pub(crate) chunk: Chunk,
    /// The function's given name (or `None` if it is anonymous).
    pub name: Option<Symbol>,
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

    /// Appends one [`Constant`] to this chunk, returning the index of the constant within
    /// [`Self::constants`].
    pub fn emit_constant(&mut self, constant: Constant) -> ConstantIndex {
        self.constants.insert(constant)
    }

    /// Appends one [`Function`] to this chunk, returning the index of the function within
    /// [`Self::functions`].
    pub fn emit_function(&mut self, function: Function) -> FunctionIndex {
        let index = self.functions.len();
        self.functions.push(Rc::new(function));
        FunctionIndex(u8::try_from(index).unwrap())
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
        let offset = to - base - 3;

        let [a, b] = offset.to_ne_bytes();
        self.code[base as usize + 1] = a;
        self.code[base as usize + 2] = b;
    }

    /// Returns the length, or current position of the chunk.
    pub fn position(&mut self) -> usize {
        self.code.len()
    }

    /// Returns the left span corresponding to the given code offset.
    pub fn span_at(&self, offset: InstructionPointer) -> Span {
        let i = self.spans.partition_point(|&(start, _)| start <= offset.0);

        self.spans[i - 1].1
    }
}

/// Disassembles a [`Function`] into a format suitable for debugging.
pub fn disassemble(f: &Function) {
    println!(
        "== {} ==",
        f.name.unwrap_or(Interner::intern("<anonymous>"))
    );

    let chunk = &f.chunk;

    println!("constants: {:?}", chunk.constants);
    println!("upvalues:  {:?}", f.upvalues);
    println!("functions:");
    for function in &chunk.functions {
        disassemble(function);
    }
    println!();

    let mut offset = InstructionPointer(0);

    while offset.0 < chunk.code.len() {
        let (instruction, next) = Instruction::decode(&chunk.code, offset);
        let span = chunk.span_at(offset);

        println!(
            "{:0>4} ({: >2}..{: <2})  {instruction}",
            offset, span.start, span.end
        );

        offset = next;
    }

    println!(
        "== end {} ==",
        f.name.unwrap_or(Interner::intern("<anonymous>"))
    );
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fn")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish()
    }
}

impl Index<InstructionPointer> for Chunk {
    type Output = u8;

    fn index(&self, index: InstructionPointer) -> &Self::Output {
        &self.code[index.0]
    }
}
