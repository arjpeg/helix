use self::{
    cursor::Cursor,
    error::LexerError,
    token::{CommandType, KeywordKind, OperatorKind, Token, TokenKind},
};

mod cursor;
pub mod error;
pub mod span;
pub mod token;

/// Struct used to lex a source file.
pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    input: &'a str,
}

impl Lexer<'_> {
    /// Creates a new lexer from a source file.
    pub fn new(content: &str) -> Lexer {
        Lexer {
            cursor: Cursor::new(content),
            input: content,
        }
    }

    /// Lexes the source file, and returns
    /// a vector of tokens.
    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        if self.cursor.peek() == Some('#') {
            let command = self.lex_command()?;

            tokens.push(Token::new(
                (0..self.cursor.pos()).into(),
                TokenKind::Command(command),
            ));

            return Ok(tokens);
        }

        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }

        Ok(tokens
            .iter()
            .filter(|t| t.token_kind != TokenKind::Whitespace)
            .cloned()
            .collect())
    }

    /// Generates the next token from the source file.
    fn next_token(&mut self) -> Result<Option<Token>, LexerError> {
        let start = self.cursor.pos();
        let c = self.cursor.advance();

        let kind = match c {
            // Whitespace
            Some(c) if c.is_ascii_whitespace() => {
                self.cursor.advance_while(|c| c.is_ascii_whitespace());

                TokenKind::Whitespace
            }

            // Numbers
            Some(c) if c.is_ascii_digit() => {
                // Advance the cursor to the end of the number.
                self.lex_number()?;

                let lexeme = &self.input[start..self.cursor.pos()];

                TokenKind::Number(lexeme.parse().unwrap())
            }

            // Operators
            Some('+') => TokenKind::Operator(OperatorKind::Plus),
            Some('-') => TokenKind::Operator(OperatorKind::Minus),
            Some('*') => TokenKind::Operator(OperatorKind::Star),
            Some('/') => TokenKind::Operator(OperatorKind::Slash),
            Some('^') => TokenKind::Operator(OperatorKind::Pow),

            Some('!') => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::Operator(OperatorKind::NotEquals)
                } else {
                    TokenKind::Operator(OperatorKind::Not)
                }
            }

            // Comparisons
            Some('=') => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::Operator(OperatorKind::Equals)
                } else {
                    TokenKind::Operator(OperatorKind::Assign)
                }
            }

            Some('<') => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::Operator(OperatorKind::LessThanOrEqual)
                } else {
                    TokenKind::Operator(OperatorKind::LessThan)
                }
            }

            Some('>') => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::Operator(OperatorKind::GreaterThanOrEqual)
                } else {
                    TokenKind::Operator(OperatorKind::GreaterThan)
                }
            }

            Some('&') => {
                if self.cursor.peek() == Some('&') {
                    self.cursor.advance();
                    TokenKind::Operator(OperatorKind::And)
                } else {
                    return Err(LexerError::UnknownSymbol {
                        range: (start..self.cursor.pos()).into(),
                    });
                }
            }

            Some('|') => {
                if self.cursor.peek() == Some('|') {
                    self.cursor.advance();
                    TokenKind::Operator(OperatorKind::Or)
                } else {
                    return Err(LexerError::UnknownSymbol {
                        range: (start..self.cursor.pos()).into(),
                    });
                }
            }

            // Parentheses
            Some('(') => TokenKind::LeftParen,
            Some(')') => TokenKind::RightParen,

            // Identifiers
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.cursor
                    .advance_while(|c| c.is_ascii_alphanumeric() || c == '_');

                let lexeme = &self.input[start..self.cursor.pos()];

                match KeywordKind::get_keyword(lexeme) {
                    Some(keyword) => TokenKind::Keyword(keyword),
                    None => TokenKind::Identifier {
                        name: lexeme.to_string(),
                    },
                }
            }

            // Anything else
            Some(_) => {
                self.cursor.advance_while(|c| !c.is_ascii_whitespace());

                return Err(LexerError::UnknownSymbol {
                    range: (start..self.cursor.pos()).into(),
                });
            }

            // End of file
            None => return Ok(None),
        };

        Ok(Some(Token::new((start..self.cursor.pos()).into(), kind)))
    }

    /// Lexes a number from the source file.
    fn lex_number(&mut self) -> Result<(), LexerError> {
        let start = self.cursor.pos() - 1;

        self.cursor.advance_while(|c| c.is_ascii_digit());

        if self.cursor.peek() == Some('.') {
            self.cursor.advance();

            self.cursor.advance_while(|c| c.is_ascii_digit());

            if self.cursor.peek() == Some('.') {
                self.cursor
                    .advance_while(|c| c.is_ascii_digit() || c == '.');

                return Err(LexerError::TooManyDots {
                    range: (start..self.cursor.pos()).into(),
                });
            };
        };

        Ok(())
    }

    /// Lexes a command
    fn lex_command(&mut self) -> Result<CommandType, LexerError> {
        self.cursor.advance();

        let start = self.cursor.pos();

        self.cursor.advance_while(|c| c.is_ascii_alphabetic());

        let lexeme = &self.input[start..self.cursor.pos()];

        if let Some(command) = CommandType::get_command(lexeme) {
            Ok(command)
        } else {
            Err(LexerError::UnknownCommand {
                range: (start..self.cursor.pos()).into(),
            })
        }
    }
}
