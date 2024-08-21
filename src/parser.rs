use crate::{
    ast::NodeKind,
    cursor::Cursor,
    error::{Error, ParserError, Result},
    token::*,
};

pub struct Parser {
    /// A cursor over the [`tokens`].
    cursor: Cursor<std::vec::IntoIter<Token>>,
    /// A list of all the [`Token`]s being parsed into the AST.
    tokens: Vec<Token>,
}

impl Parser {
    /// Creates a new [`Parser`].
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.clone(),
            cursor: Cursor::new(tokens.into_iter()),
        }
    }

    pub fn parse(mut self) -> Result<ASTNode> {
        self.equality()
    }

    /// comparison (("==" | "!=") comparison)*
    fn equality(&mut self) -> Result<ASTNode> {
        self.reduce_binary_operators(
            Self::comparison,
            &[BinaryOperator::Equals, BinaryOperator::NotEquals],
        )
    }

    /// term ((">" | ">=" | "<" | "<=") term)*
    fn comparison(&mut self) -> Result<ASTNode> {
        self.reduce_binary_operators(
            Self::term,
            &[
                BinaryOperator::LessThan,
                BinaryOperator::LessThanEquals,
                BinaryOperator::GreaterThan,
                BinaryOperator::GreaterThanEquals,
            ],
        )
    }

    /// factor (("+" | "-") factor)*
    fn term(&mut self) -> Result<ASTNode> {
        self.reduce_binary_operators(Self::factor, &[BinaryOperator::Plus, BinaryOperator::Minus])
    }

    /// unary (("*" | "/") unary)*
    fn factor(&mut self) -> Result<ASTNode> {
        self.reduce_binary_operators(
            Self::unary,
            &[BinaryOperator::Multiply, BinaryOperator::Divide],
        )
    }

    /// ("+" | "-")* unary | atom
    fn unary(&mut self) -> Result<ASTNode> {
        let token = self.consume()?;

        match token.kind {
            TokenKind::BinaryOperator(op) => {
                if let Some(op) = UnaryOperator::from_operator(op) {
                    let kind = NodeKind::UnaryOp {
                        operator: op,
                        operand: Box::new(self.unary()?),
                    };

                    let span = token.span.start..self.tokens[self.cursor.pos - 1].span.end;

                    Ok(ASTNode::new(kind, Span::new(span, token.span.source)))
                } else {
                    Err(Error {
                        span: token.span,
                        kind: ParserError::InvalidUnaryOperator(op).into(),
                    })
                }
            }

            _ => self.atom(),
        }
    }

    /// int | float | "(" expression ")"
    fn atom(&mut self) -> Result<ASTNode> {
        let token = self.consume()?;

        println!("{}", token);

        let kind = match token.kind {
            TokenKind::Float(lit) => NodeKind::Float(lit),
            TokenKind::Integer(lit) => NodeKind::Integer(lit),

            TokenKind::Keyword(keyword) => match keyword {
                Keyword::True => NodeKind::Boolean(true),
                Keyword::False => NodeKind::Boolean(false),
            },

            TokenKind::Identifier(ref ident) => NodeKind::Identifier(ident.clone()),

            TokenKind::Parenthesis(Parenthesis {
                kind: ParenthesisKind::Round,
                opening: Opening::Open,
            }) => {
                let expr = self.equality()?;

                if !matches!(
                    self.cursor.peek(),
                    Some(Token {
                        kind: TokenKind::Parenthesis(Parenthesis {
                            kind: ParenthesisKind::Round,
                            opening: Opening::Close
                        }),
                        ..
                    })
                ) {
                    let token = self.consume()?;

                    return Err(Error {
                        span: token.span,
                        kind: ParserError::MismatchedParenthesis(token).into(),
                    });
                }

                self.cursor.advance();

                return Ok(expr);
            }

            TokenKind::Parenthesis(Parenthesis {
                kind: ParenthesisKind::Round,
                opening: Opening::Close,
            }) => {
                return Err(Error {
                    span: token.span,
                    kind: ParserError::MismatchedParenthesis(token).into(),
                })
            }

            _ => {
                return Err(Error {
                    span: token.span,
                    kind: ParserError::UnexpectedToken(token).into(),
                })
            }
        };

        Ok(ASTNode::new(kind, token.span))
    }

    fn reduce_binary_operators<F>(
        &mut self,
        reducer: F,
        operators: &[BinaryOperator],
    ) -> Result<ASTNode>
    where
        F: Fn(&mut Self) -> Result<ASTNode>,
    {
        let mut lhs = reducer(self)?;

        panic!();

        while let Some(token) = self.cursor.peek().cloned() {
            dbg!(&token);

            let Some(op) = BinaryOperator::from_token_kind(&token.kind) else { break; };

            if !operators.contains(&op) {
                break;
            }

            dbg!(self.cursor.advance());

            let rhs = reducer(self)?;

            let span = lhs.span.start..rhs.span.end;

            lhs = ASTNode::new(
                NodeKind::BinaryOp {
                    operator: op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                Span::new(span, token.span.source),
            );
        }

        Ok(lhs)
    }

    fn consume(&mut self) -> Result<Token> {
        dbg!(self.cursor.advance().ok_or(Error {
            span: {
                let last = self.tokens.last().unwrap();
                Span::new(last.span.end - 1..last.span.end, last.span.source)
            },
            kind: ParserError::UnexpectedEndOfFile.into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use slotmap::{DefaultKey, Key};

    use crate::{lexer::Lexer, program::Source};

    use super::*;

    fn parse(source: &str) -> Result<NodeKind> {
        let tokens = Lexer::new(
            DefaultKey::null(),
            &Source {
                name: "<test>".to_string(),
                content: source.to_string(),
            },
        )
        .tokenize()
        .expect("test case did not tokenize properly");

        Parser::new(&tokens).parse().map(|node| node.kind)
    }

    #[test]
    fn test_literals() {
        assert!(matches!(parse("1"), Ok(NodeKind::Integer(1))));
        assert!(matches!(parse("555"), Ok(NodeKind::Integer(555))));

        assert!(
            matches!(parse("23.11"), Ok(NodeKind::Float(f)) if (f - 23.11).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn test_unary_operators() {
        let Ok(NodeKind::UnaryOp { operator: UnaryOperator::Minus, operand }) = parse("-20") else {
            panic!();
        };

        assert_eq!(operand.kind, NodeKind::Integer(20));

        let Ok(NodeKind::UnaryOp { operator: UnaryOperator::Minus, operand }) = parse("--20") else {
            panic!();
        };

        assert!(matches!(
            operand.kind,
            NodeKind::UnaryOp {
                operator: UnaryOperator::Minus,
                ..
            }
        ));
    }
}
