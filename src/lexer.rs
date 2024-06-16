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
    pub fn advance_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool,
    {
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

            c if c.is_ascii_digit() => match self.tokenize_number() {
                Ok(kind) => kind,
                Err(error) => return Some(Err(error)),
            },

            // anything else
            c => {
                if let Some(op) = Operator::from_char(c) {
                    self.cursor.advance();

                    return Some(Ok(Token::new(
                        TokenKind::Operator(op),
                        Span::new(start..self.cursor.byte_pos, self.source.index),
                    )));
                }

                self.cursor.advance_while(|c| !c.is_whitespace());

                let range = start..self.cursor.byte_pos;

                return Some(Err(Error {
                    span: Span::new(range.clone(), 0),
                    kind: match c {
                        '.' => LexerError::MalformedNumber(self.source.content[range].to_string()),
                        _ => LexerError::UnknownSymbol(self.source.content[range].to_string()),
                    }
                    .into(),
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

    /// Consumes a floating point literal or an integer literal.
    /// Note that numbers such as `.123` are not supported.
    fn tokenize_number(&mut self) -> Result<TokenKind, Error> {
        let start = self.cursor.byte_pos;

        let mut dot_count = 0;

        self.cursor.advance_while(|c| c.is_ascii_digit());

        while let Some('.') = self.cursor.peek() {
            self.cursor.advance();
            self.cursor.advance_while(|c| c.is_ascii_digit());

            dot_count += 1;
        }

        let range = start..self.cursor.byte_pos;
        let range_str = &self.source.content[range.clone()];

        match dot_count {
            0 => Ok(TokenKind::Integer(range_str.parse().unwrap())),
            1 => Ok(TokenKind::Float(range_str.parse().unwrap())),
            _ => Err(Error {
                span: Span::new(range.clone(), self.source.index),
                kind: LexerError::MalformedNumber(self.source.content[range].to_string()).into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ErrorKind;

    use super::*;

    fn tokenize(source: &str) -> Result<Vec<Token>, Error> {
        Lexer::new(&Source {
            name: "<test>".to_string(),
            content: source.to_string(),
            index: 0,
        })
        .tokenize()
    }

    #[test]
    fn test_whitespace() {
        let source = "  \t\n  ";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_numbers() {
        let source = "123 555 2.222";
        let mut tokens = tokenize(source).unwrap().into_iter();

        assert_eq!(tokens.clone().len(), 3);

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: TokenKind::Integer(123),
                ..
            })
        ));

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: TokenKind::Integer(555),
                ..
            })
        ));

        assert!(match tokens.next() {
            Some(Token {
                kind: TokenKind::Float(c),
                ..
            }) if (c - 2.222).abs() < f64::EPSILON => true,
            _ => false,
        });
    }

    #[test]
    fn test_malformed_number() {
        let source = "123.456.789";
        let error = tokenize(source).unwrap_err();

        assert!(matches!(
            error.kind,
            ErrorKind::Lexer(LexerError::MalformedNumber(_))
        ));
    }
}
