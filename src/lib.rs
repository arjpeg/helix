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
    vm::{VM, value::Value},
};

pub mod compiler;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod vm;

/// Manages the lifetime of program and REPL evaluation.
pub struct Engine {
    /// The registered [`SourceHandle`]s, along with their optimized [`Chunk`]s.
    chunks: HashMap<SourceHandle, Chunk>,

    /// The running virtual machine, shared across evaluations.
    /// TODO: allow for deferred chunk initialization
    #[allow(dead_code)]
    vm: (),
}

impl Engine {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            vm: (),
        }
    }

    /// Parses a [`Source`] file as a complete helix program, making it ready for evaluation.
    pub fn register_program(
        &mut self,
        source: SourceHandle,
    ) -> Result<(), Vec<Spanned<Box<dyn Error>>>> {
        let tokens = collect_errors(Tokenizer::new(SourceMap::get(source)))?;
        let ast = Parser::new(tokens).parse_source().map_err(|errors| {
            errors
                .into_iter()
                .map(box_error)
                .collect::<Vec<Spanned<_>>>()
        })?;

        self.chunks.insert(source, compile_program(ast));

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

        self.chunks.insert(source, compile_program(ast));

        Ok(())
    }

    /// Executes an input [`SourceHandle`], blocking until completion.
    /// Panics if the [Source] was not already registered.
    pub fn execute(
        &mut self,
        source: SourceHandle,
    ) -> Result<Option<Value>, Spanned<Box<dyn Error>>> {
        let chunk = self.chunks.get(&source).unwrap().clone();
        disassemble(&chunk);

        VM::new(chunk).execute().map_err(box_error)
    }
}

/// Converts an `Spanned<E: impl Error>` into a `Spanned<Box<dyn Error>>`
pub fn box_error(error: Spanned<impl Error + 'static>) -> Spanned<Box<dyn Error>> {
    error.map(|e| Box::new(e) as _)
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
