use std::str::Chars;

use slotmap::DefaultKey;
use unicode_xid::UnicodeXID;

use crate::{
    cursor::Cursor,
    error::{Error, LexerError, Result},
    program::Source,
    token::*,
};

/// Converts a string into a list of tokens.
pub struct Lexer<'a> {
    /// The cursor over the source code.
    cursor: Cursor<Chars<'a>>,
    /// The source file being tokenized.
    source: &'a Source,
    /// The key of the source file, used to give tokens spans.
    key: DefaultKey,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer from a string.
    pub fn new(key: DefaultKey, source: &'a Source) -> Self {
        Self {
            cursor: Cursor::new(source.content.chars()),
            key,
            source,
        }
    }

    /// Starts the tokenization process.
    pub fn tokenize(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next()? {
            match token {
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
    fn next(&mut self) -> Result<Option<Token>> {
        let start = self.cursor.pos;

        let next = match self.cursor.peek() {
            Some(c) => c,
            None => return Ok(None),
        };

        let kind = match next {
            c if c.is_whitespace() => self.skip_whitespace(),

            c if c.is_ascii_digit() => self.tokenize_number()?,

            c if c.is_xid_start() => self.tokenize_identifier(),

            c if c.is_operator_start() => {
                let next = self
                    .cursor
                    .advance()
                    .expect("found peek'ed char, should be valid to advance");

                let operator = Operator::from_chars(next, self.cursor.peek().copied())
                    .expect("operator should be valid as first char sequence was valid start");

                if operator.is_two_char() {
                    self.cursor.advance();
                }

                TokenKind::Operator(operator)
            }

            c if c.is_parenthesis() => {
                let paren = Parenthesis::from_char(*next).expect("parenthesis should be valid");
                self.cursor.advance();

                TokenKind::Parenthesis(paren)
            }

            _ => {
                self.cursor.advance_while(|c| !c.is_whitespace());
                let span = Span::new(start..self.cursor.pos, self.key);

                return Err(Error {
                    span,
                    kind: LexerError::UnknownSymbol(self.source[span].to_string()).into(),
                });
            }
        };

        let end = self.cursor.pos;

        Ok(Some(Token {
            kind,
            span: Span::new(start..end, self.key),
        }))
    }

    /// Skips whitespace characters.
    fn skip_whitespace(&mut self) -> TokenKind {
        self.cursor.advance_while(|c| c.is_whitespace());
        TokenKind::Whitespace
    }

    /// Consumes an identifier
    fn tokenize_identifier(&mut self) -> TokenKind {
        let start = self.cursor.pos;
        self.cursor.advance_while(|c| c.is_xid_continue());
        let end = self.cursor.pos;

        TokenKind::Identifier(self.source.content[start..end].to_owned())
    }

    /// Consumes a floating point literal or an integer literal.
    fn tokenize_number(&mut self) -> Result<TokenKind> {
        let start = self.cursor.pos;

        let mut dot_count = 0;

        self.cursor.advance_while(|c| c.is_ascii_digit());

        while let Some('.') = self.cursor.peek() {
            self.cursor.advance();
            self.cursor.advance_while(|c| c.is_ascii_digit());

            dot_count += 1;
        }

        let span = Span::new(start..self.cursor.pos, self.key);
        let range_str = &self.source[span];

        match dot_count {
            0 => Ok(TokenKind::Integer(range_str.parse().unwrap())),
            1 => Ok(TokenKind::Float(range_str.parse().unwrap())),
            _ => Err(Error {
                span,
                kind: LexerError::MalformedNumber(self.source[span].to_string()).into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use slotmap::Key;

    use crate::error::ErrorKind;

    use super::*;

    fn tokenize(source: &str) -> Result<Vec<Token>> {
        Lexer::new(
            DefaultKey::null(),
            &Source {
                name: "<test>".to_string(),
                content: source.to_string(),
            },
        )
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

    #[test]
    fn test_operators() {
        use crate::token::Operator::*;
        use TokenKind::*;

        let source = "23 * -1 + && !3";
        let mut tokens = tokenize(source).unwrap().into_iter();

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: Integer(23),
                ..
            })
        ));

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: Operator(Multiply),
                ..
            })
        ));

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: Operator(Minus),
                ..
            })
        ));

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: Integer(1),
                ..
            })
        ));

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: Operator(Plus),
                ..
            })
        ));

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: Operator(And),
                ..
            })
        ));

        assert!(matches!(
            tokens.next(),
            Some(Token {
                kind: Operator(Not),
                ..
            })
        ));
    }
}
