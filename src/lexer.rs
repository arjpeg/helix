//! Performs lexical analysis to convert a string into a stream of tokens.

use std::str::Chars;

use crate::errors::TokenizerError;
use crate::tokens::*;

/// A cursor that keeps track of the current position in the input string.
struct Cursor<'a> {
    /// The input string.
    input: Chars<'a>,

    /// The current position of the cursor.
    byte_pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars(),
            byte_pos: 0,
        }
    }

    /// Peek at the next character in the input string.
    /// Note that .clone isn't very expensive for chars.
    pub fn peek(&self) -> Option<char> {
        self.input.clone().next()
    }

    /// Advance the cursor by one character.
    pub fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        self.byte_pos += c.map(|c| c.len_utf8()).unwrap_or_default();

        c
    }

    /// Skip the current character if it matches the given predicate.
    pub fn skip_while(&mut self, predicate: fn(char) -> bool) {
        while matches!(self.peek(), Some(c) if predicate(c)) {
            self.advance();
        }
    }
}

pub struct Tokenizer<'a> {
    input: &'a str,
    cursor: Cursor<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            cursor: Cursor::new(input),
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens = Vec::new();

        // use the next_token method to get the next token
        while let Some(token) = self.next_token()? {
            if token.kind == TokenKind::Whitespace {
                continue;
            }

            tokens.push(token);
        }

        // add an EOF token to the end of the token stream
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::from(self.cursor.byte_pos..self.cursor.byte_pos),
        });

        Ok(tokens)
    }

    /// Skip whitespace characters.
    fn skip_whitespace(&mut self) {
        self.cursor.skip_while(|c| c.is_whitespace());
    }

    fn skip_number_and_decimal_point(&mut self) {
        self.cursor.skip_while(|c| c.is_ascii_digit() || c == '.');
    }

    /// Consumes a number literal.
    fn number(&mut self) -> Result<(), TokenizerError> {
        let mut num_decimal_points = 0;
        let start = self.cursor.byte_pos - 1;

        if &self.input[start..start + 1] == "." {
            num_decimal_points += 1;
        }

        self.cursor.skip_while(|c| c.is_ascii_digit());

        if self.cursor.peek() == Some('.') {
            if num_decimal_points > 0 {
                self.skip_number_and_decimal_point();

                return Err(TokenizerError::TooManyDecimalPoints {
                    span: (start..self.cursor.byte_pos).into(),
                });
            }

            self.cursor.advance();
            self.cursor.skip_while(|c| c.is_ascii_digit());

            num_decimal_points += 1;
        }

        if self.cursor.peek() == Some('.') && num_decimal_points >= 1 {
            self.skip_number_and_decimal_point();

            return Err(TokenizerError::TooManyDecimalPoints {
                span: (start..self.cursor.byte_pos).into(),
            });
        }

        Ok(())
    }

    fn identifer(&mut self) {
        self.cursor
            .skip_while(|c| c.is_ascii_alphanumeric() || c == '_');
    }

    /// Returns the next token in the input string.
    fn next_token(&mut self) -> Result<Option<Token>, TokenizerError> {
        let start = self.cursor.byte_pos;
        let c = self.cursor.advance();

        let kind: TokenKind = match c {
            // Whitespace
            Some(c) if c.is_whitespace() => {
                self.skip_whitespace();
                TokenKind::Whitespace
            }

            // Numbers
            Some(c) if c.is_ascii_digit() || c == '.' => {
                self.number()?;

                let number = &self.input[start..self.cursor.byte_pos];

                TokenKind::Number(number.parse().unwrap())
            }

            // Operators
            Some('+') => TokenKind::Operator(OperatorKind::Plus),
            Some('-') => TokenKind::Operator(OperatorKind::Minus),
            Some('*') => TokenKind::Operator(OperatorKind::Star),
            Some('/') => TokenKind::Operator(OperatorKind::Slash),

            // Parenthesis
            Some('(') => TokenKind::OpenParenthesis,
            Some(')') => TokenKind::CloseParenthesis,

            // Any other character
            Some(_) => {
                self.identifer();

                return Err(TokenizerError::UnexpectedIdentifier {
                    span: Span::from(start..self.cursor.byte_pos),
                    identifier: self.input[start..self.cursor.byte_pos].to_string(),
                });
            }

            // End of input
            None => return Ok(None),
        };

        let end = self.cursor.byte_pos;
        let span = Span::from(start..end);

        Ok(Some(Token { kind, span }))
    }
}
