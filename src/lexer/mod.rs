use std::rc::Rc;

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
    file: Rc<str>,
}

impl Lexer<'_> {
    /// Creates a new lexer from a source file.
    pub fn new(content: &str, file: Rc<str>) -> Lexer {
        Lexer {
            cursor: Cursor::new(content),
            input: content,
            file,
        }
    }

    /// Lexes the source file, and returns
    /// a vector of tokens.
    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        if self.cursor.peek() == Some('#') {
            let command = self.lex_command()?;

            tokens.push(Token::new(
                (0..self.cursor.pos(), Rc::clone(&self.file)).into(),
                TokenKind::Command(command),
            ));

            return Ok(tokens);
        }

        while let Some(token) = self.next_token()? {
            if token.token_kind == TokenKind::RightBrace {
                tokens.push(Token::new(
                    (self.cursor.pos()..self.cursor.pos(), Rc::clone(&self.file)).into(),
                    TokenKind::Newline,
                ));
            }

            tokens.push(token);
        }

        // Push one last newline token
        tokens.push(Token::new(
            (self.cursor.pos()..self.cursor.pos(), Rc::clone(&self.file)).into(),
            TokenKind::Newline,
        ));

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
            // Newline or semicolon
            Some('\n' | ';') => TokenKind::Newline,

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

            // Strings
            Some('"') => {
                self.cursor.advance_while(|c| c != '"');

                if self.cursor.peek() != Some('"') {
                    return Err(LexerError::UnterminatedString {
                        range: (start..self.cursor.pos(), Rc::clone(&self.file)).into(),
                    });
                }

                let lexeme = &self.input[start + 1..self.cursor.pos()];

                self.cursor.advance();

                TokenKind::String(lexeme.to_string())
            }

            // Operators
            Some('+') => TokenKind::Operator(OperatorKind::Plus),
            Some('-') => TokenKind::Operator(OperatorKind::Minus),
            Some('*') => TokenKind::Operator(OperatorKind::Star),
            Some('/') => TokenKind::Operator(OperatorKind::Slash),
            Some('^') => TokenKind::Operator(OperatorKind::Power),

            Some('!') => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::Operator(OperatorKind::NotEquals)
                } else {
                    TokenKind::Operator(OperatorKind::Bang)
                }
            }

            Some(',') => TokenKind::Comma,

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
                        range: (start..self.cursor.pos(), Rc::clone(&self.file)).into(),
                    });
                }
            }

            Some('|') => {
                if self.cursor.peek() == Some('|') {
                    self.cursor.advance();
                    TokenKind::Operator(OperatorKind::Or)
                } else {
                    return Err(LexerError::UnknownSymbol {
                        range: (start..self.cursor.pos(), Rc::clone(&self.file)).into(),
                    });
                }
            }

            // Parentheses
            Some('(') => TokenKind::LeftParen,
            Some(')') => TokenKind::RightParen,

            // Braces
            Some('{') => TokenKind::LeftBrace,
            Some('}') => TokenKind::RightBrace,

            // Identifiers
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.cursor
                    .advance_while(|c| c.is_ascii_alphanumeric() || c == '_');

                let lexeme = &self.input[start..self.cursor.pos()];

                KeywordKind::get_keyword(lexeme).map_or_else(
                    || TokenKind::Identifier {
                        name: lexeme.to_string(),
                    },
                    |keyword| TokenKind::Keyword(keyword),
                )
            }

            // Anything else
            Some(_) => {
                self.cursor.advance_while(|c| !c.is_ascii_whitespace());

                return Err(LexerError::UnknownSymbol {
                    range: (start..self.cursor.pos(), Rc::clone(&self.file)).into(),
                });
            }

            // End of file
            None => return Ok(None),
        };

        Ok(Some(Token::new(
            (start..self.cursor.pos(), Rc::clone(&self.file)).into(),
            kind,
        )))
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
                    range: (start..self.cursor.pos(), Rc::clone(&self.file)).into(),
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
                range: (start..self.cursor.pos(), Rc::clone(&self.file)).into(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::lexer::{
        error::LexerError,
        token::{CommandType, KeywordKind, OperatorKind, TokenKind},
        Lexer,
    };

    #[test]
    fn test_empty() {
        let mut lexer = Lexer::new("", Rc::from(""));
        assert_eq!(lexer.lex().unwrap().len(), 1);
    }

    #[test]
    fn test_whitespace() {
        let mut lexer = Lexer::new(" \t\n\r", Rc::from(""));
        assert_eq!(lexer.lex().unwrap().len(), 1);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("123 456.789 0.1 32", Rc::from(""));
        let tokens = lexer.lex().unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_kind, TokenKind::Number(123.0));
        assert_eq!(tokens[1].token_kind, TokenKind::Number(456.789));
        assert_eq!(tokens[2].token_kind, TokenKind::Number(0.1));
        assert_eq!(tokens[3].token_kind, TokenKind::Number(32.0));
    }

    #[test]
    fn test_invalid_numbers() {
        let mut lexer = Lexer::new("123.123.456.789", Rc::from(""));
        let tokens = lexer.lex();

        assert!(tokens.is_err() && matches!(tokens.unwrap_err(), LexerError::TooManyDots { .. }));
    }

    #[test]
    fn test_commands() {
        let mut lexer = Lexer::new("#quit", Rc::from(""));
        let tokens = lexer.lex().unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_kind, TokenKind::Command(CommandType::Quit));
    }

    #[test]
    fn test_invalid_commands() {
        let mut lexer = Lexer::new("#invalid", Rc::from(""));
        let tokens = lexer.lex();

        assert!(
            tokens.is_err() && matches!(tokens.unwrap_err(), LexerError::UnknownCommand { .. })
        );
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+-*/^", Rc::from(""));
        let tokens = lexer.lex().unwrap();

        assert_eq!(tokens.len(), 6);
        assert_eq!(
            tokens[0].token_kind,
            TokenKind::Operator(OperatorKind::Plus)
        );
        assert_eq!(
            tokens[1].token_kind,
            TokenKind::Operator(OperatorKind::Minus)
        );
        assert_eq!(
            tokens[2].token_kind,
            TokenKind::Operator(OperatorKind::Star)
        );
        assert_eq!(
            tokens[3].token_kind,
            TokenKind::Operator(OperatorKind::Slash)
        );
        assert_eq!(
            tokens[4].token_kind,
            TokenKind::Operator(OperatorKind::Power)
        );
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("foo bar baz", Rc::from(""));
        let tokens = lexer.lex().unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(
            tokens[0].token_kind,
            TokenKind::Identifier {
                name: "foo".to_string()
            }
        );
        assert_eq!(
            tokens[1].token_kind,
            TokenKind::Identifier {
                name: "bar".to_string()
            }
        );
        assert_eq!(
            tokens[2].token_kind,
            TokenKind::Identifier {
                name: "baz".to_string()
            }
        );
        assert_eq!(tokens[3].token_kind, TokenKind::Newline);
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("let fn if else", Rc::from(""));
        let tokens = lexer.lex().unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_kind, TokenKind::Keyword(KeywordKind::Let));
        assert_eq!(
            tokens[1].token_kind,
            TokenKind::Keyword(KeywordKind::Function)
        );
        assert_eq!(tokens[2].token_kind, TokenKind::Keyword(KeywordKind::If));
        assert_eq!(tokens[3].token_kind, TokenKind::Keyword(KeywordKind::Else));
        assert_eq!(tokens[4].token_kind, TokenKind::Newline);
    }

    #[test]
    fn test_invalid_symbols() {
        let mut lexer = Lexer::new("foo bar baz $", Rc::from(""));
        let tokens = lexer.lex();

        assert!(tokens.is_err() && matches!(tokens.unwrap_err(), LexerError::UnknownSymbol { .. }));
    }

    #[test]
    fn test_invalid_unicode() {
        let mut lexer = Lexer::new("foo bar baz \u{1F4A9}", Rc::from(""));
        let tokens = lexer.lex();

        assert!(tokens.is_err() && matches!(tokens.unwrap_err(), LexerError::UnknownSymbol { .. }));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"foo\" \"bar\" \"baz\"", Rc::from(""));
        let tokens = lexer.lex().unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token_kind, TokenKind::String("foo".to_string()));
        assert_eq!(tokens[1].token_kind, TokenKind::String("bar".to_string()));
        assert_eq!(tokens[2].token_kind, TokenKind::String("baz".to_string()));
    }

    #[test]
    fn test_unterminated_strings() {
        let mut lexer = Lexer::new("\"foo", Rc::from(""));
        let tokens = lexer.lex();

        assert!(
            tokens.is_err() && matches!(tokens.unwrap_err(), LexerError::UnterminatedString { .. })
        );
    }

    #[test]
    fn test_multi_character_lexemes() {
        let mut lexer = Lexer::new("= == ! != < <= > >= && ||", Rc::from(""));
        let tokens = lexer.lex().unwrap();

        assert_eq!(tokens.len(), 11);
        assert_eq!(
            tokens[0].token_kind,
            TokenKind::Operator(OperatorKind::Assign)
        );
        assert_eq!(
            tokens[1].token_kind,
            TokenKind::Operator(OperatorKind::Equals)
        );

        assert_eq!(
            tokens[2].token_kind,
            TokenKind::Operator(OperatorKind::Bang)
        );
        assert_eq!(
            tokens[3].token_kind,
            TokenKind::Operator(OperatorKind::NotEquals)
        );

        assert_eq!(
            tokens[4].token_kind,
            TokenKind::Operator(OperatorKind::LessThan)
        );
        assert_eq!(
            tokens[5].token_kind,
            TokenKind::Operator(OperatorKind::LessThanOrEqual)
        );

        assert_eq!(
            tokens[6].token_kind,
            TokenKind::Operator(OperatorKind::GreaterThan)
        );
        assert_eq!(
            tokens[7].token_kind,
            TokenKind::Operator(OperatorKind::GreaterThanOrEqual)
        );

        assert_eq!(tokens[8].token_kind, TokenKind::Operator(OperatorKind::And));
        assert_eq!(tokens[9].token_kind, TokenKind::Operator(OperatorKind::Or));
    }
}
