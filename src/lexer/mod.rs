pub mod error;
pub mod token;

use unicode_xid::UnicodeXID;

use crate::{
    lexer::{
        error::{Result, TokenizationError},
        token::{CharTokenExt, OpKind, Token},
    },
    source::{Source, Span, Spanned},
};

/// Converts source code into a stream of [`Token`]s.
pub struct Tokenizer {
    /// The source code being tokeinzed.
    source: Source,
    /// The current byte position within the `source`.
    cursor: usize,
}

impl Tokenizer {
    /// Creates a new [`Tokenizer`].
    pub fn new(source: Source) -> Self {
        Self { source, cursor: 0 }
    }

    /// Returns the remaining characters to be tokenized.
    fn remaining(&self) -> &str {
        &self.source.content[self.cursor..]
    }

    /// Peeks at the next character without advancing the cursor.
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Advances the cursor forward by one character, returning it if not at eof.
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.cursor += c.len_utf8();
        Some(c)
    }

    /// Advances the cursor past all whitespace characters.
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.advance();
        }
    }

    /// Advances the cursor while the predicate is met, returning the input and span consumed.
    fn advance_while(&mut self, predicate: impl Fn(&char) -> bool) -> Span {
        let start = self.cursor;

        while matches!(self.peek(), Some(c) if predicate(&c)) {
            self.advance();
        }

        Span::new(self.source, start..self.cursor)
    }

    /// Tokenizes a single symbol (keyword or identifier).
    fn next_symbol(&mut self) -> Spanned<Token> {
        let span = self.advance_while(|c| c.is_xid_continue());
        Spanned::wrap(Token::Symbol(span.text()), span)
    }

    /// Tokenizes a single operator (may span multiple characters).
    fn next_operator(&mut self) -> Spanned<Token> {
        let start = self.cursor;
        let operator = OpKind::try_from((self.advance().unwrap(), self.peek())).unwrap();

        if operator.len() == 2 {
            self.advance();
        }

        let span = Span::new(self.source, start..self.cursor);

        Spanned::wrap(Token::Operator(operator), span)
    }

    /// Tokenizes a single integer literal.
    fn next_integer(&mut self) -> Result<Spanned<Token>> {
        let span = self.advance_while(|c| !c.is_whitespace());
        let literal = span.text();

        literal
            .parse()
            .map(|n| Spanned::wrap(Token::Integer(n), span))
            .map_err(|_| Spanned::wrap(TokenizationError::InvalidIntegerLiteral(literal), span))
    }
}

impl Iterator for Tokenizer {
    type Item = Result<Spanned<Token>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        Some(match self.peek()? {
            c if c == '_' || c.is_xid_start() => Ok(self.next_symbol()),

            c if c.is_ascii_digit() => self.next_integer(),

            c if c.is_operator_start() => Ok(self.next_operator()),

            _ => {
                let span = self.advance_while(|c| !c.is_whitespace());

                Err(Spanned::wrap(
                    TokenizationError::UnknownSymbol(span.text()),
                    span,
                ))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    use error::TokenizationError::*;
    use token::Token::*;

    fn tokenize(input: &'static str) -> Vec<Result<Spanned<Token>>> {
        Tokenizer::new(Source {
            content: input,
            path: Path::new(""),
        })
        .collect()
    }

    fn strip_spans(tokens: Vec<Result<Spanned<Token>>>) -> Vec<Result<Token, TokenizationError>> {
        tokens
            .into_iter()
            .map(|res| {
                res.map(|spanned| spanned.value)
                    .map_err(|spanned| spanned.value)
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_empty() {
        assert_eq!(tokenize(""), vec![]);
        assert_eq!(tokenize("   \t\n\n\n"), vec![]);
    }

    #[test]
    fn test_integers() {
        assert_eq!(
            strip_spans(tokenize("23 55 1")),
            vec![Ok(Integer(23)), Ok(Integer(55)), Ok(Integer(1))]
        );

        assert_eq!(
            strip_spans(tokenize("  001 045x asdb22")),
            vec![
                Ok(Integer(1)),
                Err(InvalidIntegerLiteral("045x")),
                Ok(Symbol("asdb22"))
            ]
        );
    }

    #[test]
    fn test_symbols() {
        assert_eq!(
            strip_spans(tokenize("helloo\n_world a_23")),
            vec![
                Ok(Symbol("helloo")),
                Ok(Symbol("_world")),
                Ok(Symbol("a_23"))
            ]
        );
    }

    #[test]
    fn test_unknown() {
        assert_eq!(
            strip_spans(tokenize("$1@2 ~ad")),
            vec![Err(UnknownSymbol("$1@2")), Err(UnknownSymbol("~ad")),]
        );
    }
}
