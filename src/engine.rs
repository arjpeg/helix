use std::collections::HashMap;

use crate::{
    error::Error,
    interpreter::Interpreter,
    lexer::Tokenizer,
    parser::{Parser, ast::Statement},
    source::{Source, Spanned},
};

/// Manages the lifecycle of the interpreter and other global state.
#[derive(Debug, Clone)]
pub struct Engine {
    /// A list of the loaded source files, along with their ASTs.
    sources: HashMap<Source, Spanned<Statement>>,
    /// The main interpreter, responsible for excecuting the `sources`.
    interpreter: Interpreter,
}

impl Engine {
    /// Creates a new [`Engine`].
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            interpreter: Interpreter::new(),
        }
    }

    /// Registers a source file into the engine, parsing it and making it ready for execution.
    pub fn register(&mut self, source: Source) -> Result<Source, Spanned<Error>> {
        let tokens = Tokenizer::new(source).collect::<Result<Vec<_>, _>>()?;
        let ast = Parser::new(tokens).parse_source()?;

        self.sources.insert(source, ast);

        Ok(source)
    }

    /// Excecutes the interpreter on the loaded source file, blocking until the program terminates.
    pub fn excecute(&mut self, source: Source) {
        self.interpreter
            .excecute(self.sources.get(&source).unwrap());
    }
}
