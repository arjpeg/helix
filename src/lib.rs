use std::collections::HashMap;

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
    pub fn register_program(&mut self, source: Source) -> Result<Source, Spanned<Error>> {
        let tokens = Tokenizer::new(source).collect::<Result<Vec<_>, _>>()?;
        let ast = Parser::new(tokens).parse_source()?;

        self.asts.insert(source, ast);

        Ok(source)
    }

    /// Parses a [Source] file as a REPL input, making it ready for evaluation.
    pub fn register_repl(&mut self, source: Source) -> Result<Source, Spanned<Error>> {
        let tokens = Tokenizer::new(source).collect::<Result<Vec<_>, _>>()?;
        let ast = Parser::new(tokens).parse_repl()?;

        self.asts.insert(source, ast);

        Ok(source)
    }

    /// Executes an input [Source], blocking until completetion.
    /// Panics if the [Source] was not already registered.
    pub fn execute(&mut self, source: Source) -> Result<Option<Value>, Spanned<Error>> {
        Ok(self.interpreter.execute(self.asts.get(&source).unwrap())?)
    }
}

/// Parses a [`Source`] as a complete program, returning the corresponding AST.
pub fn parse_program(source: Source) -> Result<Spanned<Statement>, Spanned<Error>> {
    let tokens = Tokenizer::new(source).collect::<Result<Vec<_>, _>>()?;
    Ok(Parser::new(tokens).parse_source()?)
}

/// Parses a [`Source`] as a repl input, returning the corresponding AST.
pub fn parse_repl(source: Source) -> Result<Spanned<Statement>, Spanned<Error>> {
    let tokens = Tokenizer::new(source).collect::<Result<Vec<_>, _>>()?;
    Ok(Parser::new(tokens).parse_repl()?)
}
