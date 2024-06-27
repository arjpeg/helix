use std::slice::Iter;

use crate::{
    cursor::Cursor,
    error::{Error, ParserError},
    token::{Operator, Token, TokenKind, UnaryOperator},
};

type ASTNode = crate::ast::Node;

type Result = std::result::Result<crate::ast::Node, crate::error::Error>;

pub struct Parser<'a> {
    /// A cursor over the tokens
    cursor: Cursor<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            cursor: Cursor::new(tokens.iter()),
        }
    }

    pub fn parse(mut self) -> Result {
        self.arithmetic_expression()
    }

    /// term ((PLUS|MINUS) term)*
    fn arithmetic_expression(&mut self) -> Result {
        self.reduce_binary_operators(Self::term, &[Operator::Plus, Operator::Minus])
    }

    /// factor ((MUL|DIV) factor) *
    fn term(&mut self) -> Result {
        self.reduce_binary_operators(Self::factor, &[Operator::Multiply, Operator::Divide])
    }

    /// (PLUS|MINUS)* atom
    fn factor(&mut self) -> Result {
        // TODO: remove unwrap - add into ParserError for Option?
        let token = self.cursor.peek().unwrap();

        match token.kind {
            TokenKind::Operator(op) => {
                if let Some(op) = UnaryOperator::from_operator(op) {
                    self.cursor.advance();

                    Ok(ASTNode::UnaryOp {
                        operator: op,
                        operand: Box::new(self.factor()?),
                    })
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

    /// int | float
    fn atom(&mut self) -> Result {
        let token = self.cursor.advance().unwrap();

        let node = match token.kind {
            TokenKind::Float(lit) => Ok(ASTNode::Float(lit)),
            TokenKind::Integer(lit) => Ok(ASTNode::Integer(lit)),

            _ => todo!("{token:?}"),
        };

        node
    }

    fn reduce_binary_operators<F>(&mut self, reducer: F, operators: &[Operator]) -> Result
    where
        F: Fn(&mut Self) -> Result,
    {
        let mut lhs = reducer(self)?;

        while let Some(token) = self.cursor.peek() {
            dbg!(&lhs, &token);

            let Some(op) = Operator::from_token_kind(token.kind) else {
                println!("STOPPING");
                break;
            };

            if !operators.contains(&op) {
                break;
            }

            self.cursor.advance();

            let rhs = reducer(self)?;

            lhs = ASTNode::BinaryOp {
                lhs: Box::new(lhs),
                operator: op,
                rhs: Box::new(rhs),
            }
        }

        Ok(lhs)
    }
}
