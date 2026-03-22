use crate::{
    error::Error,
    interpreter::{Interpreter, value::Value},
    lexer::Tokenizer,
    parser::Parser,
    source::{Source, Spanned},
};

pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod source;

/// Execute a [`Source`] as a complete program, blocking until completion.
pub fn run_program(source: Source) -> Result<(), Spanned<Error>> {
    let tokens = Tokenizer::new(source).collect::<Result<Vec<_>, _>>()?;
    let ast = Parser::new(tokens).parse_source()?;

    dbg!(&ast);

    let _ = Interpreter::new().excecute(&ast)?;

    Ok(())
}

/// Execute a [`Source`] as a repl input, blocking until completion.
pub fn run_repl(source: Source) -> Result<Option<Value>, Spanned<Error>> {
    let tokens = Tokenizer::new(source).collect::<Result<Vec<_>, _>>()?;
    let ast = Parser::new(tokens).parse_repl()?;

    Ok(Interpreter::new().excecute(&ast)?)
}
