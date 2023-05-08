use errors::{HelixError, TokenizerError};
use input::prompt;
use parser::Parser;

use crate::lexer::Tokenizer;

use owo_colors::OwoColorize;

// Module imports
mod errors;
mod input;
mod lexer;
mod nodes;
mod parser;
mod tokens;

fn main() {
    loop {
        let input = prompt();

        let tokenizer = Tokenizer::new(&input);
        let tokens = tokenizer.tokenize();

        match tokens {
            Err(error) => {
                println!(
                    "{}",
                    format_error(HelixError::TokenizerError(error), &input)
                );

                continue;
            }
            Ok(_) => {}
        }

        // create a parser
        let mut parser = Parser::new(tokens.unwrap());

        // parse the tokens
        let ast = parser.parse();

        match ast {
            Err(error) => {
                println!("{}", format_error(HelixError::ParserError(error), &input));

                continue;
            }
            Ok(_) => {
                println!("{:#?}", ast.unwrap());
            }
        }
    }
}

/// A function to format an error message.
fn format_error(error: HelixError, input: &str) -> String {
    use TokenizerError::*;

    let (msg, span) = match error {
        HelixError::TokenizerError(error) => match error {
            UnexpectedIdentifier { span, identifier } => {
                (format!("Unexpected identifier: '{}'", identifier), span)
            }

            TooManyDecimalPoints { span } => {
                (format!("Too many decimal points in number literal"), span)
            }
        },
        HelixError::ParserError(error) => match error {
            errors::ParserError::ExpectedBinaryOperator(span) => {
                (format!("Expected a binary operator"), span)
            }
            errors::ParserError::ExpectedNewExpression(span) => {
                (format!("Expected a new expression"), span)
            }
            errors::ParserError::UnclosedParenthesis(span) => {
                (format!("Unclosed parenthesis"), span)
            }
            errors::ParserError::UnmatchedClosingParenthesis(span) => {
                (format!("Unmatched closing parenthesis"), span)
            }
            errors::ParserError::UnexpectedToken {
                span,
                expected,
                found,
            } => (
                format!(
                    "Expected {}, but found a token of type: {:?}",
                    expected, found
                ),
                span.clone(),
            ),
            errors::ParserError::UnexpectedEof(span) => (format!("Unexpected end of input"), span),
        },
    };

    let explanation_line = format!("{}: {}", "error".red().bold(), msg);

    let src_line = format!("      {}", input);

    let padding = " ".repeat(input[0..span.start].chars().count());
    let underline = "^".repeat(input[span].chars().count());
    let src_underline = format!("      {}{}", padding, underline);

    format!(
        "\
{}
{}{}",
        explanation_line,
        src_line.white(),
        src_underline
    )
}
