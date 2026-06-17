use std::{collections::HashMap, error::Error};

use itertools::{Either, Itertools};

use crate::{
    compiler::{
        chunk::{Chunk, disassemble},
        compile_program,
    },
    lexer::Tokenizer,
    parser::Parser,
    source::{SourceHandle, SourceMap, Spanned},
    vm::{VM, globals::Globals, value::Value},
};

pub mod compiler;
pub mod error;
pub mod interner;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod vm;

/// Whether or not the running chunk is disassembled for debugging purposes or not.
pub const DEBUG_DISASSEMBLE_CHUNK: bool = false;

/// Manages the lifetime of program and REPL evaluation.
pub struct Engine {
    /// The registered [`SourceHandle`]s, along with their optimized [`Chunk`]s.
    chunks: HashMap<SourceHandle, Chunk>,

    /// The running virtual machine, shared across evaluations.
    vm: VM,

    /// The shared state of global variables for all sources.
    globals: Globals,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            vm: VM::new(),
            globals: Globals::new(),
        }
    }

    /// Parses a [`Source`] file as a complete helix program, making it ready for evaluation.
    pub fn register_program(
        &mut self,
        source: SourceHandle,
    ) -> Result<(), Vec<Spanned<Box<dyn Error>>>> {
        let tokens = collect_errors(Tokenizer::new(SourceMap::get(source)))?;
        let ast = Parser::new(tokens).parse_source().map_err(box_vec_error)?;
        let (chunk, globals) = compile_program(ast, &self.globals).map_err(box_vec_error)?;

        self.chunks.insert(source, chunk);
        self.globals = globals;

        Ok(())
    }

    /// Parses a [`Source`] file as a REPL input, making it ready for evaluation.
    pub fn register_repl(
        &mut self,
        source: SourceHandle,
    ) -> Result<(), Vec<Spanned<Box<dyn Error>>>> {
        let tokens = collect_errors(Tokenizer::new(SourceMap::get(source)))?;
        let ast = Parser::new(tokens)
            .parse_repl()
            .map_err(|e| vec![box_error(e)])?;
        let (chunk, globals) = compile_program(ast, &self.globals).map_err(box_vec_error)?;

        self.chunks.insert(source, chunk);
        self.globals = globals;

        Ok(())
    }

    /// Executes an input [`SourceHandle`], blocking until completion.
    /// Panics if the [Source] was not already registered.
    pub fn execute(
        &mut self,
        source: SourceHandle,
    ) -> Result<Option<Value>, Spanned<Box<dyn Error>>> {
        let chunk = self.chunks.get(&source).unwrap();

        if DEBUG_DISASSEMBLE_CHUNK {
            disassemble(chunk);
        }

        self.vm.globals = self.globals.snapshot();

        match self.vm.execute(chunk) {
            Ok(value) => {
                // synchronize global states
                self.globals = self.vm.globals.snapshot();
                Ok(value)
            }

            Err(e) => Err(box_error(e)),
        }
    }
}

/// Converts an `Spanned<E: impl Error>` into a `Spanned<Box<dyn Error>>`
fn box_error(error: Spanned<impl Error + 'static>) -> Spanned<Box<dyn Error>> {
    error.map(|e| Box::new(e) as _)
}

/// Converts an `Spanned<Vec<Spanned<E: impl Error>>` into a `Spanned<Vec<Box<dyn Error>>>`
fn box_vec_error(errors: Vec<Spanned<impl Error + 'static>>) -> Vec<Spanned<Box<dyn Error>>> {
    errors.into_iter().map(box_error).collect_vec()
}

/// Separates an `Iterator<Item = Result<T, E>> into Result<Vec<T>, Vec<Box<dyn Error>>>`
fn collect_errors<T, E: Into<Spanned<impl Error + 'static>>>(
    iter: impl Iterator<Item = Result<T, E>>,
) -> Result<Vec<T>, Vec<Spanned<Box<dyn Error>>>> {
    let (oks, errs): (Vec<_>, Vec<_>) = iter.into_iter().partition_map(|r| match r {
        Ok(t) => Either::Left(t),
        Err(e) => Either::Right(box_error(e.into())),
    });

    if errs.is_empty() { Ok(oks) } else { Err(errs) }
}
