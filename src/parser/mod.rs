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
                let res = self.parse_statements()?;

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

    /// Parses a list of statements
    pub fn parse_statements(&mut self) -> ParserResult<AstNode> {
        let start = self.pos;

        let mut statements = Vec::new();

        while self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }

        let end = self.pos;

        match statements.len() {
            1 => Ok(statements[0].clone()),

            _ => Ok(AstNode {
                kind: AstNodeKind::Block {
                    expressions: statements,
                },
                span: Span::new(start, end),
            }),
        }
    }

    /// Parses a statement
    fn parse_statement(&mut self) -> ParserResult<AstNode> {
        // Check against the current token

        match self.peek().unwrap().token_kind {
            TokenKind::Keyword(keyword) => match keyword {
                KeywordKind::Let => self.parse_assignment(),
                KeywordKind::If => self.parse_if(),
                KeywordKind::Print => self.parse_print(),
                KeywordKind::While => self.parse_while(),
                KeywordKind::Function => self.parse_function(),

                _ => {
                    return Err(ParserError::UnexpectedToken {
                        expected: "a statement".to_string(),
                        found: self.peek().unwrap().clone(),
                    })
                }
            },

            TokenKind::LeftBrace => self.parse_block(),

            _ => self.parse_expr(),
        }
    }

    /// Parses a block. (LBRACE STATEMENT* RBRACE)
    fn parse_block(&mut self) -> ParserResult<AstNode> {
        let start = self.pos;

        self.expect(&[TokenKind::LeftBrace])?;
        self.advance();

        let mut statements = Vec::new();

        while !matches!(
            self.peek(),
            Some(Token {
                token_kind: TokenKind::RightBrace,
                ..
            }),
        ) {
            statements.push(self.parse_statement()?);
        }

        self.expect(&[TokenKind::RightBrace])?;
        self.advance();

        let end = self.pos;

        Ok(AstNode {
            kind: AstNodeKind::Block {
                expressions: statements,
            },
            span: Span::new(start, end),
        })
    }

    /// Parses an assignment statement. (LET IDENT (OP)? ASSIGN EXPR)
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

        let binary_assign_op = match self.clone().peek() {
            Some(t) => match &t.token_kind {
                TokenKind::Operator(op) => match op {
                    OperatorKind::Plus
                    | OperatorKind::Minus
                    | OperatorKind::Star
                    | OperatorKind::Slash
                    | OperatorKind::Power => {
                        self.advance();
                        Some(op.clone())
                    }
                    _ => None,
                },
                _ => None,
            },
            None => None,
        };

        self.expect(&[TokenKind::Operator(OperatorKind::Assign)])?;
        self.advance();

        let expr = self.parse_expr()?;

        let end = self.pos;

        match binary_assign_op {
            Some(op) => Ok(AstNode {
                kind: AstNodeKind::Assignment {
                    name: ident.clone(),
                    value: Box::new(AstNode {
                        kind: AstNodeKind::BinaryExpression {
                            op,
                            lhs: Box::new(AstNode {
                                kind: AstNodeKind::VariableReference(ident),
                                span: Span::new(start, end),
                            }),
                            rhs: Box::new(expr),
                        },
                        span: Span::new(start, end),
                    }),
                },
                span: Span::new(start, end),
            }),

            None => Ok(AstNode {
                kind: AstNodeKind::Assignment {
                    name: ident,
                    value: Box::new(expr),
                },
                span: Span::new(start, end),
            }),
        }
    }

    /// Parses a print statement. (PRINT EXPR)
    fn parse_print(&mut self) -> ParserResult<AstNode> {
        let start = self.pos;

        self.advance();

        let found_left_paren = matches!(
            self.peek(),
            Some(Token {
                token_kind: TokenKind::LeftParen,
                ..
            }),
        );

        if found_left_paren {
            self.advance();
        }

        let expr = self.parse_expr()?;

        if found_left_paren {
            self.expect(&[TokenKind::RightParen])?;
            self.advance();
        }

        let end = self.pos;

        Ok(AstNode {
            kind: AstNodeKind::Print {
                expression: Box::new(expr),
            },
            span: Span::new(start, end),
        })
    }

    /// Parses a function declaration. (FUNCTION IDENT LBRACE STATEMENT* RBRACE)
    fn parse_function(&mut self) -> ParserResult<AstNode> {
        let start = self.pos;

        self.expect(&[TokenKind::Keyword(KeywordKind::Function)])?;
        self.advance();

        let name = match self.peek() {
            Some(Token {
                token_kind: TokenKind::Identifier { name },
                ..
            }) => name.clone(),
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: "an identifier as the function name".to_string(),
                    found: self.peek().unwrap().clone(),
                })
            }
        };
        self.advance();

        self.expect(&[TokenKind::LeftParen])?;
        self.advance();

        let mut params = Vec::new();

        while let Some(Token {
            token_kind: TokenKind::Identifier { name },
            ..
        }) = self.clone().peek()
        {
            params.push(name.clone());
            self.advance();

            self.expect(&[TokenKind::Comma, TokenKind::RightParen])?;

            if matches!(
                self.clone().peek(),
                Some(Token {
                    token_kind: TokenKind::RightParen,
                    ..
                })
            ) {
                break;
            }

            self.advance();
        }

        // Just a sanity check, as the above loop should always break
        self.expect(&[TokenKind::RightParen])?;
        self.advance();

        let body = self.parse_block()?;

        let end = self.pos;

        Ok(AstNode {
            kind: AstNodeKind::FunctionDefinition {
                params,
                name: name.clone(),
                body: Box::new(body),
            },
            span: Span::new(start, end),
        })
    }

    /// Parses an if statement. (IF EXPR BLOCK (else_statement)?)
    fn parse_if(&mut self) -> ParserResult<AstNode> {
        let start = self.pos;
        self.advance();

        let condition = self.parse_expr()?;
        let body = self.parse_block()?;

        let else_body = match self.peek() {
            Some(Token {
                token_kind: TokenKind::Keyword(KeywordKind::Else),
                ..
            }) => Some(self.parse_else()?),
            _ => None,
        };

        let end = self.pos;

        Ok(AstNode {
            kind: AstNodeKind::If {
                condition: Box::new(condition),
                body: Box::new(body),
                else_branch: else_body.map(Box::new),
            },
            span: Span::new(start, end),
        })
    }

    /// Parses an else statement. (ELSE BLOCK) | (ELSE if_statement)
    fn parse_else(&mut self) -> ParserResult<AstNode> {
        let start = self.pos;
        self.expect(&[TokenKind::Keyword(KeywordKind::Else)])?;
        self.advance();

        let else_body = match self.peek() {
            Some(Token {
                token_kind: TokenKind::Keyword(KeywordKind::If),
                ..
            }) => self.parse_if()?,
            _ => self.parse_block()?,
        };

        let end = self.pos;

        Ok(AstNode {
            kind: AstNodeKind::Else {
                body: Box::new(else_body),
            },
            span: Span::new(start, end),
        })
    }

    /// Parses a while statement. (WHILE EXPR BLOCK)
    fn parse_while(&mut self) -> ParserResult<AstNode> {
        let start = self.pos;
        self.advance();

        let condition = self.parse_expr()?;
        let body = self.parse_block()?;

        let end = self.pos;

        Ok(AstNode {
            kind: AstNodeKind::While {
                condition: Box::new(condition),
                body: Box::new(body),
            },
            span: Span::new(start, end),
        })
    }

    /// Parses an expression. (COMP_EXPR) (AND|OR COMP_EXPR)*
    fn parse_expr(&mut self) -> ParserResult<AstNode> {
        self.parse_binary_expr(
            Self::parse_comp_expr,
            &[OperatorKind::And, OperatorKind::Or],
            None::<fn(&mut Self) -> ParserResult<AstNode>>,
        )
    }

    /// Parses a comparison expression. (ARITH_EXPR) (LT|GT|LTE|GTE|EQ|NEQ  ARITH_EXPR)*
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

            TokenKind::String(string) => {
                self.advance();

                Ok(AstNode {
                    kind: AstNodeKind::StringLiteral(string),
                    span: token.span,
                })
            }

            TokenKind::LeftBrace => self.parse_block(),

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

                self.expect(&[TokenKind::RightParen])?;
                self.advance();

                Ok(expr)
            }

            TokenKind::RightParen => Err(ParserError::UnmatchedClosingParen { paren: token }),

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
    fn expect(&mut self, kinds: &[TokenKind]) -> ParserResult<()> {
        let kinds_str = {
            let mut kinds_str = String::new();

            for (idx, kind) in kinds.iter().enumerate() {
                if idx == kinds.len() - 1 {
                    kinds_str.push_str(&format!("{:?}", kind));
                } else {
                    kinds_str.push_str(&format!("{:?}, ", kind));
                }
            }

            kinds_str
        };

        match self.peek() {
            Some(token) if kinds.contains(&token.token_kind) => Ok(()),

            Some(token) => Err(ParserError::UnexpectedToken {
                expected: format!("a {}", kinds_str),
                found: token.clone(),
            }),
            None => Err(ParserError::UnexpectedEof {
                expected: format!("a {:?}", kinds_str),
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

    #[test]
    fn test_conditional_exprs() {
        let tokens = Lexer::new("1 + 3 > 2 && 1 < 2").lex().unwrap();
        let mut parser = Parser::new(tokens);

        #[allow(illegal_floating_point_literal_pattern)]
        if let AstNodeKind::BinaryExpression { lhs, rhs, op } = parser.parse().unwrap().kind {
            assert!(matches!(
                lhs.kind,
                AstNodeKind::BinaryExpression {
                    lhs: _,
                    rhs: _,
                    op: OperatorKind::GreaterThan
                }
            ));
            assert!(matches!(
                rhs.kind,
                AstNodeKind::BinaryExpression {
                    lhs: _,
                    rhs: _,
                    op: OperatorKind::LessThan
                }
            ));
            assert!(matches!(op, OperatorKind::And));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_conditional_exprs_with_parentheses() {
        let tokens = Lexer::new("(1 + 3 > 2) && (1 < 2)").lex().unwrap();
        let mut parser = Parser::new(tokens);

        #[allow(illegal_floating_point_literal_pattern)]
        if let AstNodeKind::BinaryExpression { lhs, rhs, op } = parser.parse().unwrap().kind {
            assert!(matches!(
                lhs.kind,
                AstNodeKind::BinaryExpression {
                    lhs: _,
                    rhs: _,
                    op: OperatorKind::GreaterThan
                }
            ));
            assert!(matches!(
                rhs.kind,
                AstNodeKind::BinaryExpression {
                    lhs: _,
                    rhs: _,
                    op: OperatorKind::LessThan
                }
            ));
            assert!(matches!(op, OperatorKind::And));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_if_statement() {
        let tokens = Lexer::new("if (1 + 3 > 2) { 1 }").lex().unwrap();
        let mut parser = Parser::new(tokens);

        #[allow(illegal_floating_point_literal_pattern)]
        if let AstNodeKind::If {
            condition,
            body,
            else_branch,
        } = parser.parse().unwrap().kind
        {
            assert!(matches!(
                condition.kind,
                AstNodeKind::BinaryExpression {
                    lhs: _,
                    rhs: _,
                    op: OperatorKind::GreaterThan
                }
            ));
            assert!(matches!(body.kind, AstNodeKind::Block { .. }));
            assert!(matches!(else_branch, None));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_if_with_many_else() {
        let tokens = Lexer::new(
            r#"
if (1 + 3 > 2) {
    1
} else if (1 < 2) {
    2
} else {
    3
}
        "#,
        )
        .lex()
        .unwrap();

        let mut parser = Parser::new(tokens);

        #[allow(illegal_floating_point_literal_pattern)]
        if let AstNodeKind::If {
            condition,
            body,
            else_branch,
        } = parser.parse().unwrap().kind
        {
            assert!(matches!(
                condition.kind,
                AstNodeKind::BinaryExpression {
                    lhs: _,
                    rhs: _,
                    op: OperatorKind::GreaterThan
                }
            ));
            assert!(matches!(body.kind, AstNodeKind::Block { .. }));

            if let Some(else_branch) = else_branch {
                if let AstNodeKind::Else { body } = else_branch.kind {
                    assert!(matches!(body.kind, AstNodeKind::If { .. }));
                } else {
                    assert!(false);
                }
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }
}
