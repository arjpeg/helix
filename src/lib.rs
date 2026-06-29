use std::{collections::HashMap, rc::Rc};

use itertools::{Either, Itertools};

use crate::{
    compiler::{
        chunk::{Function, disassemble},
        compile_program,
    },
    error::Error,
    lexer::Tokenizer,
    parser::Parser,
    source::{SourceHandle, SourceMap, Spanned},
    vm::{VM, globals::Globals, stdlib, value::Value},
};

pub mod compiler;
pub mod error;
pub mod interner;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod vm;

/// Whether or not the running script is disassembled for debugging purposes or not.
pub const DEBUG_DISASSEMBLE_SCRIPT: bool = false;

/// Manages the lifetime of program and REPL evaluation.
pub struct Engine {
    /// The registered [`SourceHandle`]s, along with their optimized [`Function`]s.
    scripts: HashMap<SourceHandle, Rc<Function>>,

    /// The running virtual machine, shared across evaluations.
    vm: VM,

    /// The shared state of global variables for all sources.
    globals: Globals,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
            vm: VM::new(),
            globals: stdlib::default_environment(),
        }
    }

    /// Parses a [`Source`] file as a complete helix program, making it ready for evaluation.
    pub fn register_program(&mut self, source: SourceHandle) -> Result<(), Vec<Spanned<Error>>> {
        let tokens = collect_errors(Tokenizer::new(SourceMap::get(source)))?;
        let ast = Parser::new(tokens).parse_source().map_err(flatten_errors)?;
        let (chunk, globals) = compile_program(ast, &self.globals).map_err(flatten_errors)?;

        self.scripts.insert(source, Rc::new(chunk));
        self.globals = globals;

        Ok(())
    }

    /// Parses a [`Source`] file as a REPL input, making it ready for evaluation.
    pub fn register_repl(&mut self, source: SourceHandle) -> Result<(), Vec<Spanned<Error>>> {
        let tokens = collect_errors(Tokenizer::new(SourceMap::get(source)))?;
        let ast = Parser::new(tokens)
            .parse_repl()
            .map_err(|e| vec![e.into()])?;

        let (chunk, globals) = compile_program(ast, &self.globals).map_err(flatten_errors)?;

        self.scripts.insert(source, Rc::new(chunk));
        self.globals = globals;

        Ok(())
    }

    /// Executes an input [`SourceHandle`], blocking until completion.
    /// Panics if the [Source] was not already registered.
    pub fn execute(&mut self, source: SourceHandle) -> Result<Option<Value>, Spanned<Error>> {
        let script = Rc::clone(&self.scripts.get(&source).unwrap());

        if DEBUG_DISASSEMBLE_SCRIPT {
            disassemble(&script);
        }

        self.vm.globals = self.globals.snapshot();

        match self.vm.execute(script) {
            Ok(value) => {
                // synchronize global states
                self.globals = self.vm.globals.snapshot();
                Ok(value)
            }

            Err(e) => Err(e.into()),
        }
    }
}

/// Flattens a list of specific concrete errors into a list of [`Error`]s.
fn flatten_errors(errors: Vec<Spanned<impl Into<Error>>>) -> Vec<Spanned<Error>> {
    errors
        .into_iter()
        .map(|spanned| spanned.map(Into::into))
        .collect()
}

/// Separates an `Iterator<Item = Result<T, E>> into Result<Vec<T>, Vec<Spanned<Error>>>`
fn collect_errors<T, E>(
    iter: impl Iterator<Item = Result<T, E>>,
) -> Result<Vec<T>, Vec<Spanned<Error>>>
where
    E: Into<Spanned<Error>>,
{
    let (oks, errs): (Vec<_>, Vec<_>) = iter.into_iter().partition_map(|r| match r {
        Ok(t) => Either::Left(t),
        Err(e) => Either::Right(e.into()),
    });

    if errs.is_empty() { Ok(oks) } else { Err(errs) }
}
