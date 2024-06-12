use std::{iter::Peekable, str::Chars};

use crate::{
    error::{Error, LexerError},
    program::Source,
    token::*,
};

/// A cursor that keeps track of the current position in the input string.
struct Cursor<'a> {
    /// The input string.
    input: Peekable<Chars<'a>>,

    /// The current position of the cursor.
    byte_pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            byte_pos: 0,
        }
    }

    /// Peek at the next character in the input string.
    pub fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    /// Advance the cursor by one character.
    pub fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        self.byte_pos += c.map(|c| c.len_utf8()).unwrap_or(0);

        c
    }

    /// Skip the current character if it matches the given predicate.
    pub fn advance_while(&mut self, predicate: fn(char) -> bool) {
        while matches!(self.peek(), Some(c) if predicate(c)) {
            self.advance();
        }
    }
}

/// Converts a string into a list of tokens.
pub struct Lexer<'a> {
    /// The cursor over the source code.
    cursor: Cursor<'a>,
    /// The source file being tokenized.
    source: &'a Source,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer from a string.
    pub fn new(source: &'a Source) -> Self {
        Self {
            cursor: Cursor::new(&source.content),
            source,
        }
    }

    /// Starts the tokenization process.
    pub fn tokenize(mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next() {
            match token? {
                Token {
                    kind: TokenKind::Whitespace,
                    ..
                } => continue,

                token => tokens.push(token),
            }
        }

        Ok(tokens)
    }

    /// Advances the lexer by one token.
    fn next(&mut self) -> Option<Result<Token, Error>> {
        let start = self.cursor.byte_pos;

        let kind = match self.cursor.peek()? {
            c if c.is_whitespace() => {
                self.skip_whitespace();
                TokenKind::Whitespace
            }

            c if c.is_digit(10) => {
                self.tokenize_integer();

                TokenKind::Integer(
                    self.source.content[start..self.cursor.byte_pos]
                        .parse()
                        .unwrap(),
                )
            }

            _ => {
                self.cursor.advance_while(|c| !c.is_whitespace());

                let range = start..self.cursor.byte_pos;

                return Some(Err(Error {
                    span: Span::new(range.clone(), 0),
                    kind: LexerError::UnknownSymbol(self.source.content[range].to_string()).into(),
                }));
            }
        };

        let end = self.cursor.byte_pos;

        Some(Ok(Token::new(
            kind,
            Span::new(start..end, self.source.index),
        )))
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) {
        self.cursor.advance_while(char::is_whitespace);
    }

    /// Consumes an integer literal.
    fn tokenize_integer(&mut self) {
        self.cursor.advance_while(|c| c.is_digit(10));
    }
}
