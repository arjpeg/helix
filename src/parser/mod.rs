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
                    return Err(ParserError::UnexpectedToken {
                        expected: "end of input".to_string(),
                        found: self.peek().unwrap().clone(),
                    });
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
                OperatorKind::Plus | OperatorKind::Minus | OperatorKind::Not => {
                    let start = self.pos;
                    self.advance();

                    let expr = self.parse_expr()?;

                    Ok(AstNode {
                        kind: AstNodeKind::UnaryExpression {
                            operator: op,
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

                if self.peek().unwrap().token_kind != TokenKind::RightParen {
                    return Err(ParserError::UnexpectedToken {
                        expected: "a right parenthesis".to_string(),
                        found: self.peek().unwrap().clone(),
                    });
                }

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
                kind: AstNodeKind::BinaryExpr {
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
    /// If it is, it moves forward in the list of tokens.
    fn expect(&mut self, kind: TokenKind) -> ParserResult<()> {
        match self.peek() {
            Some(token) if token.token_kind == kind => {
                self.advance();
                Ok(())
            }
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
