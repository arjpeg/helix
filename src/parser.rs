use std::slice::Iter;

use crate::{
    cursor::Cursor,
    error::{Error, ParserError, Result},
    token::{Keyword, Operator, Token, TokenKind, UnaryOperator},
};

type ASTNode = crate::ast::Node;

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

    pub fn parse(mut self) -> Result<ASTNode> {
        self.equality()
    }

    /// comparison (("==" | "!=") comparison)*
    fn equality(&mut self) -> Result<ASTNode> {
        self.reduce_binary_operators(Self::comparison, &[Operator::Equals, Operator::NotEquals])
    }

    /// term ((">" | ">=" | "<" | "<=") term)*
    fn comparison(&mut self) -> Result<ASTNode> {
        self.reduce_binary_operators(
            Self::term,
            &[
                Operator::LessThan,
                Operator::LessThanEquals,
                Operator::GreaterThan,
                Operator::GreaterThanEquals,
            ],
        )
    }

    /// factor (("+" | "-") factor)*
    fn term(&mut self) -> Result<ASTNode> {
        self.reduce_binary_operators(Self::factor, &[Operator::Plus, Operator::Minus])
    }

    /// unary (("*" | "/") unary)*
    fn factor(&mut self) -> Result<ASTNode> {
        self.reduce_binary_operators(Self::unary, &[Operator::Multiply, Operator::Divide])
    }

    /// ("+" | "-")* unary | atom
    fn unary(&mut self) -> Result<ASTNode> {
        // TODO: remove unwrap - add into ParserError for Option?
        let token = self.cursor.peek().unwrap();

        match token.kind {
            TokenKind::Operator(op) => {
                if let Some(op) = UnaryOperator::from_operator(op) {
                    self.cursor.advance();

                    Ok(ASTNode::UnaryOp {
                        operator: op,
                        operand: Box::new(self.unary()?),
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

    /// int | float | "(" expression ")"
    fn atom(&mut self) -> Result<ASTNode> {
        let token = self.cursor.advance().unwrap();

        match token.kind {
            TokenKind::Float(lit) => Ok(ASTNode::Float(lit)),
            TokenKind::Integer(lit) => Ok(ASTNode::Integer(lit)),

            TokenKind::Keyword(keyword) => match keyword {
                Keyword::True => Ok(ASTNode::Boolean(true)),
                Keyword::False => Ok(ASTNode::Boolean(false)),
            },

            TokenKind::Identifier(ref ident) => Ok(ASTNode::Identifier(ident.clone())),

            _ => Err(Error {
                span: token.span,
                kind: ParserError::UnexpectedToken(token.clone()).into(),
            }),
        }
    }

    fn reduce_binary_operators<F>(&mut self, reducer: F, operators: &[Operator]) -> Result<ASTNode>
    where
        F: Fn(&mut Self) -> Result<ASTNode>,
    {
        let mut lhs = reducer(self)?;

        while let Some(token) = self.cursor.peek() {
            let Some(op) = Operator::from_token_kind(&token.kind) else { break; };

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

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, program::Source};

    use super::*;

    fn parse(source: &str) -> Result<ASTNode> {
        let tokens = Lexer::new(&Source {
            name: "<test>".to_string(),
            content: source.to_string(),
            index: 0,
        })
        .tokenize()
        .expect("test case did not tokenize properly");

        Parser::new(&tokens).parse()
    }

    #[test]
    fn test_literals() {
        assert!(matches!(parse("1"), Ok(ASTNode::Integer(1))));
        assert!(matches!(parse("555"), Ok(ASTNode::Integer(555))));

        assert!(
            matches!(parse("23.11"), Ok(ASTNode::Float(f)) if (f - 23.11).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn test_unary_operators() {
        let Ok(ASTNode::UnaryOp { operator: UnaryOperator::Minus, operand }) = parse("-20") else {
            panic!();
        };

        assert_eq!(*operand, ASTNode::Integer(20));

        let Ok(ASTNode::UnaryOp { operator: UnaryOperator::Minus, operand }) = parse("--20") else {
            panic!();
        };

        assert!(matches!(
            *operand,
            ASTNode::UnaryOp {
                operator: UnaryOperator::Minus,
                ..
            }
        ));
    }
}
