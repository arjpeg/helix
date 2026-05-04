use std::collections::HashMap;

use itertools::{Either, Itertools};

use crate::{
    error::Error,
    interpreter::{Interpreter, value::Value},
    lexer::Tokenizer,
    parser::{Parser, ast::Statement},
    source::{Source, Spanned},
};

pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod source;

/// Manages the lifetime of program and REPL evaluation.
pub struct Engine {
    /// The registered [Source] files, along with their parsed AST.
    asts: HashMap<Source, Spanned<Statement>>,
    /// The running interpreter, shared across evaluations.
    interpreter: Interpreter,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            asts: HashMap::new(),
            interpreter: Interpreter::new(),
        }
    }

    /// Parses a [Source] file as a complete helix program, making it ready for evaluation.
    pub fn register_program(&mut self, source: Source) -> Result<Source, Vec<Spanned<Error>>> {
        let tokens = collect_errors(Tokenizer::new(source))?;
        let ast = Parser::new(tokens).parse_source().map_err(|errors| {
            errors
                .into_iter()
                .map(|e| e.into())
                .collect::<Vec<Spanned<Error>>>()
        })?;

        self.asts.insert(source, ast);

        Ok(source)
    }

    /// Parses a [Source] file as a REPL input, making it ready for evaluation.
    pub fn register_repl(&mut self, source: Source) -> Result<Source, Vec<Spanned<Error>>> {
        let tokens = collect_errors(Tokenizer::new(source))?;
        let ast = Parser::new(tokens)
            .parse_repl()
            .map_err(|e| vec![e.into()])?;

        self.asts.insert(source, ast);

        Ok(source)
    }

    /// Executes an input [Source], blocking until completetion.
    /// Panics if the [Source] was not already registered.
    pub fn execute(&mut self, source: Source) -> Result<Option<Value>, Spanned<Error>> {
        Ok(self.interpreter.execute(self.asts.get(&source).unwrap())?)
    }
}

/// Seperates an `Iterator<Item = Result<T, E>> into Result<Vec<T>, Vec<Error>>`
fn collect_errors<T, E: Into<Spanned<Error>>>(
    iter: impl Iterator<Item = Result<T, E>>,
) -> Result<Vec<T>, Vec<Spanned<Error>>> {
    let (oks, errs): (Vec<_>, Vec<_>) = iter.into_iter().partition_map(|r| match r {
        Ok(t) => Either::Left(t),
        Err(e) => Either::Right(e.into()),
    });

    if errs.is_empty() { Ok(oks) } else { Err(errs) }
}
