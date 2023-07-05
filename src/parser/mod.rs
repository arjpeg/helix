pub mod ast;
pub mod error;

use crate::lexer::{
    span::Span,
    token::{KeywordKind, OperatorKind, Token, TokenKind},
};

use self::{
    ast::{AstNode, AstNodeKind},
    error::ParserError,
};

/// A type alias for a result from the parser.
pub type ParserResult<T> = Result<T, ParserError>;

/// Struct to represent a parser.
/// A parser takes a list of tokens and parses them into an AST.
/// The AST can then be used to interpret the code.
#[derive(Debug, Clone)]
pub struct Parser {
    /// The list of tokens to parse.
    tokens: Vec<Token>,
    /// The current position in the list of tokens.
    pos: usize,
}

impl Parser {
    /// Creates a new parser from a list of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parses the list of tokens into an AST.
    pub fn parse(&mut self) -> ParserResult<AstNode> {
        match self.tokens.len() {
            0 => Ok(AstNode {
                kind: AstNodeKind::Empty,
                span: Span::new(0, 0),
            }),
            _ => {
                let res = self.parse_statement()?;

                if self.pos != self.tokens.len() {
                    return match self.peek().unwrap().token_kind {
                        TokenKind::RightParen => Err(ParserError::UnmatchedClosingParen {
                            paren: self.peek().unwrap().clone(),
                        }),

                        _ => Err(ParserError::UnexpectedToken {
                            expected: "end of input".to_string(),
                            found: self.peek().unwrap().clone(),
                        }),
                    };
                }

                Ok(res)
            }
        }
    }

    /// Parses a statement
    fn parse_statement(&mut self) -> ParserResult<AstNode> {
        // Check against the current token
        match self.peek().unwrap().token_kind {
            TokenKind::Keyword(keyword) => match keyword {
                KeywordKind::Let => self.parse_assignment(),
            },
            _ => self.parse_expr(),
        }
    }

    /// Parses an assignment statement. (LET IDENT ASSIGN EXPR)
    fn parse_assignment(&mut self) -> ParserResult<AstNode> {
        let start = self.pos;

        self.advance();

        let ident = match self.clone().peek() {
            Some(t) => match &t.token_kind {
                TokenKind::Identifier { name } => {
                    self.advance();
                    name
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "an identifier".to_string(),
                        found: t.clone(),
                    })
                }
            },

            None => {
                return Err(ParserError::UnexpectedEof {
                    expected: "an identifier".to_string(),
                })
            }
        }
        .clone();

        self.expect(TokenKind::Operator(OperatorKind::Assign))?;
        self.advance();

        let expr = self.parse_expr()?;

        let end = self.pos;

        Ok(AstNode {
            kind: AstNodeKind::Assignment {
                name: ident,
                value: Box::new(expr),
            },
            span: Span::new(start, end),
        })
    }

    /// Parses an expression. (COMP_EXPR) (EQ|NEQ COMP_EXPR)*
    fn parse_expr(&mut self) -> ParserResult<AstNode> {
        self.parse_binary_expr(
            Self::parse_comp_expr,
            &[OperatorKind::And, OperatorKind::Or],
            None::<fn(&mut Self) -> ParserResult<AstNode>>,
        )
    }

    /// Parses a comparison expression. (ARITH_EXPR) (LT|GT|LTE|GTE ARITH_EXPR)*
    fn parse_comp_expr(&mut self) -> ParserResult<AstNode> {
        self.parse_binary_expr(
            Self::parse_arith_expr,
            &[
                OperatorKind::Equals,
                OperatorKind::NotEquals,
                OperatorKind::LessThan,
                OperatorKind::GreaterThan,
                OperatorKind::LessThanOrEqual,
                OperatorKind::GreaterThanOrEqual,
            ],
            None::<fn(&mut Self) -> ParserResult<AstNode>>,
        )
    }

    /// Parses an expression. (TERM) (PLUS|MINUS TERM)*
    fn parse_arith_expr(&mut self) -> ParserResult<AstNode> {
        self.parse_binary_expr(
            Self::parse_term,
            &[OperatorKind::Plus, OperatorKind::Minus],
            None::<fn(&mut Self) -> ParserResult<AstNode>>,
        )
    }

    /// Parses a term. (FACTOR) (MULT|DIV FACTOR)*
    fn parse_term(&mut self) -> ParserResult<AstNode> {
        self.parse_binary_expr(
            Self::parse_factor,
            &[OperatorKind::Star, OperatorKind::Slash],
            None::<fn(&mut Self) -> ParserResult<AstNode>>,
        )
    }

    /// Parses a factor. (ATOM) (POW factor)*
    fn parse_factor(&mut self) -> ParserResult<AstNode> {
        self.parse_binary_expr(
            Self::parse_atom,
            &[OperatorKind::Power],
            Some(Self::parse_factor),
        )
    }

    /// Parses an atom.
    fn parse_atom(&mut self) -> ParserResult<AstNode> {
        let token = match self.clone().peek() {
            Some(token) => token.clone(),
            None => {
                return Err(ParserError::UnexpectedEof {
                    expected: "a number literal or left parenthesis".to_string(),
                })
            }
        };

        match token.token_kind {
            TokenKind::Number(num) => {
                self.advance();

                Ok(AstNode {
                    kind: AstNodeKind::NumberLiteral(num),
                    span: token.span,
                })
            }

            TokenKind::Identifier { name } => {
                self.advance();

                Ok(AstNode {
                    kind: AstNodeKind::VariableReference(name),
                    span: token.span,
                })
            }

            TokenKind::Operator(op) => match op {
                OperatorKind::Plus | OperatorKind::Minus | OperatorKind::Bang => {
                    let start = self.pos;
                    self.advance();

                    let expr = self.parse_expr()?;

                    Ok(AstNode {
                        kind: AstNodeKind::UnaryExpression {
                            op,
                            expr: Box::new(expr),
                        },
                        span: Span::new(start, self.pos),
                    })
                }

                _ => Err(ParserError::UnexpectedToken {
                    expected: "a number literal or a left parenthesis".to_string(),
                    found: token,
                }),
            },

            TokenKind::LeftParen => {
                self.advance();

                let expr = self.parse_expr()?;

                self.expect(TokenKind::RightParen)?;
                self.advance();

                Ok(expr)
            }

            _ => Err(ParserError::UnexpectedToken {
                expected: "a number literal or a left parenthesis".to_string(),
                found: token,
            }),
        }
    }

    /// Parses a binary expression.
    /// Starts with a left hand side, then parses the operator
    /// (as long as it exists), then parses the right hand side.
    fn parse_binary_expr(
        &mut self,
        left_fn: impl Fn(&mut Self) -> ParserResult<AstNode>,
        operators: &[OperatorKind],
        right_fn: Option<impl Fn(&mut Self) -> ParserResult<AstNode>>,
    ) -> ParserResult<AstNode> {
        let mut lhs = left_fn(self)?;
        let start = lhs.span.start;

        while let Some(op) = self.clone().peek() {
            if !matches!(op.token_kind, TokenKind::Operator(op) if operators.contains(&op)) {
                break;
            }

            self.advance();

            let rhs = match right_fn.as_ref() {
                Some(right_fn) => right_fn(self)?,
                None => left_fn(self)?,
            };

            let end = rhs.span.end;

            lhs = AstNode {
                kind: AstNodeKind::BinaryExpression {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    op: match op.token_kind {
                        TokenKind::Operator(op) => op,
                        _ => unreachable!(),
                    },
                },
                span: (start..end).into(),
            };
        }

        Ok(lhs)
    }

    /// Moves forward in the list of tokens.
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Peeks at the current token.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Expects the current token to be of a certain kind.
    fn expect(&mut self, kind: TokenKind) -> ParserResult<()> {
        match self.peek() {
            Some(token) if token.token_kind == kind => Ok(()),

            Some(token) => Err(ParserError::UnexpectedToken {
                expected: format!("a {:?}", kind),
                found: token.clone(),
            }),
            None => Err(ParserError::UnexpectedEof {
                expected: format!("a {:?}", kind),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{
            token::{OperatorKind, Token, TokenKind},
            Lexer,
        },
        parser::{
            ast::{AstNode, AstNodeKind},
            error::ParserError,
            Parser,
        },
    };

    #[test]
    fn test_empty() {
        let tokens = vec![];
        let mut parser = Parser::new(tokens);

        assert!(matches!(parser.parse().unwrap().kind, AstNodeKind::Empty));
    }

    #[test]
    fn test_number() {
        let tokens = Lexer::new("123").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap().kind,
            AstNodeKind::NumberLiteral(..)
        ));
    }

    #[test]
    fn test_binary_expr() {
        let tokens = Lexer::new("1 + 2").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap().kind,
            AstNodeKind::BinaryExpression {
                // TODO: Make this work with Box::new or somehow
                //       dereference the Box.
                lhs: _,
                rhs: _,
                op: OperatorKind::Plus
            }
        ));
    }

    #[test]
    fn test_order_of_operations() {
        let tokens = Lexer::new("1 + 2 * 3").lex().unwrap();
        let mut parser = Parser::new(tokens);

        #[allow(illegal_floating_point_literal_pattern)]
        if let AstNodeKind::BinaryExpression { lhs, rhs, op } = parser.parse().unwrap().kind {
            assert!(matches!(lhs.kind, AstNodeKind::NumberLiteral(1.)));
            assert!(matches!(op, OperatorKind::Plus));

            let AstNode { kind, span: _ } = *rhs;

            if let AstNodeKind::BinaryExpression { lhs, rhs, op } = kind {
                assert!(matches!(lhs.kind, AstNodeKind::NumberLiteral(2.)));
                assert!(matches!(rhs.kind, AstNodeKind::NumberLiteral(3.)));
                assert!(matches!(op, OperatorKind::Star));
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parentheses() {
        let tokens = Lexer::new("(1 + 2) * 3").lex().unwrap();
        let mut parser = Parser::new(tokens);

        #[allow(illegal_floating_point_literal_pattern)]
        if let AstNodeKind::BinaryExpression { lhs, rhs, op } = parser.parse().unwrap().kind {
            assert!(matches!(op, OperatorKind::Star));

            let AstNode { kind, span: _ } = *lhs;

            if let AstNodeKind::BinaryExpression { lhs, rhs, op } = kind {
                assert!(matches!(lhs.kind, AstNodeKind::NumberLiteral(1.)));
                assert!(matches!(rhs.kind, AstNodeKind::NumberLiteral(2.)));
                assert!(matches!(op, OperatorKind::Plus));
            } else {
                assert!(false);
            }

            let AstNode { kind, span: _ } = *rhs;

            if let AstNodeKind::NumberLiteral(3.) = kind {
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_assignment() {
        let tokens = Lexer::new("let a = 1").lex().unwrap();
        let mut parser = Parser::new(tokens);

        #[allow(illegal_floating_point_literal_pattern)]
        if let AstNodeKind::Assignment { name, value } = parser.parse().unwrap().kind {
            assert_eq!(name, "a");
            assert!(matches!(value.kind, AstNodeKind::NumberLiteral(1.)));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_invalid_assignment() {
        let tokens = Lexer::new("a = 1 + 2").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap_err(),
            crate::parser::ParserError::UnexpectedToken { .. }
        ));
    }

    #[test]
    fn test_incomplete_bin_expr() {
        let tokens = Lexer::new("1 +").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap_err(),
            ParserError::UnexpectedEof { .. }
        ));
    }

    #[test]
    fn test_invalid_bin_expr() {
        let tokens = Lexer::new("1 + / 2").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap_err(),
            ParserError::UnexpectedToken {
                found: Token {
                    token_kind: TokenKind::Operator(OperatorKind::Slash),
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_invalid_parentheses() {
        let tokens = Lexer::new("(1 + 2").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap_err(),
            ParserError::UnexpectedEof { .. }
        ));
    }

    #[test]
    fn test_unary_expr() {
        let tokens = Lexer::new("-1").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap().kind,
            AstNodeKind::UnaryExpression {
                op: OperatorKind::Minus,
                ..
            }
        ));

        let tokens = Lexer::new("!1").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap().kind,
            AstNodeKind::UnaryExpression {
                op: OperatorKind::Bang,
                ..
            }
        ));
    }

    #[test]
    fn test_unmatched_parentheses() {
        let tokens = Lexer::new("(1 + 2))").lex().unwrap();
        let mut parser = Parser::new(tokens);

        assert!(matches!(
            parser.parse().unwrap_err(),
            ParserError::UnmatchedClosingParen { .. }
        ));
    }
}
