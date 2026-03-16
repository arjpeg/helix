use owo_colors::OwoColorize;
use thiserror::Error;

use crate::{
    interpreter::error::RuntimeError, lexer::error::TokenizationError, parser::error::ParsingError,
    source::Spanned,
};

/// All errors that can occur during program execution.
#[derive(Debug, Clone, PartialEq, Error)]
#[error(transparent)]
pub enum Error {
    Tokenization(TokenizationError),
    Parsing(ParsingError),
    Runtime(RuntimeError),
}

/// Pretty prints an error to the console.
pub fn print_error(error: Spanned<Error>) {
    // get the line that the error occured on
    let span = error.span;
    let source = span.source;

    let line_start = source.content[..span.start]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let line_end = source.content[span.end..]
        .find('\n')
        .map(|i| i + span.end)
        .unwrap_or(source.content.len());

    let line_number = source.content[..span.start]
        .chars()
        .filter(|c| *c == '\n')
        .count()
        + 1;

    let line = &source.content[line_start..line_end];

    // how far into the line span starts
    let line_offset = span.start - line_start;

    println!("{}: {}", "error".red().bold(), error.value.bold());
    println!(
        "  {}:{}",
        source.path.display().cyan().dimmed(),
        line_number.cyan().dimmed()
    );

    println!("{}    {line}", ">".black());
    println!(
        "     {repeat}{arrows}",
        repeat = " ".repeat(line_offset),
        arrows = "^".repeat(span.end - span.start)
    );
}

impl From<Spanned<TokenizationError>> for Spanned<Error> {
    fn from(value: Spanned<TokenizationError>) -> Self {
        value.map(Error::Tokenization)
    }
}

impl From<Spanned<ParsingError>> for Spanned<Error> {
    fn from(value: Spanned<ParsingError>) -> Self {
        value.map(Error::Parsing)
    }
}

impl From<Spanned<RuntimeError>> for Spanned<Error> {
    fn from(value: Spanned<RuntimeError>) -> Self {
        value.map(Error::Runtime)
    }
}
