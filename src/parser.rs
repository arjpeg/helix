//! A module to parse tokens into an AST.

use crate::{
    errors::ParserError,
    nodes::{ArithmeticExpression, BinaryOperation, ParseTree, UnaryOperation},
    tokens::{OperatorKind, Token, TokenKind},
};

/// A struct to represent a parser.
pub struct Parser {
    tokens: Vec<Token>,
}

type Result<T, E = ParserError> = std::result::Result<T, E>;

impl Parser {
    /// Create a new parser from a vector of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    /// Entry point for the parser. Uses
    /// [recursive descent parsing](https://en.wikipedia.org/wiki/Recursive_descent_parser).
    pub fn parse(&mut self) -> Result<ParseTree> {
        if self.tokens.len() == 1 {
            Ok(ParseTree::Empty)
        } else {
            let res = ParseTree::Expression(self.expression()?);

            // make sure there are no more tokens
            if let Some(next) = self.peek() {
                match next.kind {
                    TokenKind::Eof => {}
                    TokenKind::CloseParenthesis => {
                        return Err(ParserError::UnmatchedClosingParenthesis(next.span))
                    }
                    _ => return Err(ParserError::ExpectedBinaryOperator(next.span)),
                }
            }

            Ok(res)
        }
    }

    /// Parse an expression.
    fn expression(&mut self) -> Result<ArithmeticExpression> {
        self.binary_operator(Self::term, &[OperatorKind::Plus, OperatorKind::Minus])
    }

    /// Parse a term.
    fn term(&mut self) -> Result<ArithmeticExpression> {
        self.binary_operator(Self::factor, &[OperatorKind::Star, OperatorKind::Slash])
    }

    /// Parse a factor.
    fn factor(&mut self) -> Result<ArithmeticExpression> {
        // Either a number, a unary operator, or a parenthesized expression.
        let token = self
            .peek()
            .ok_or(ParserError::ExpectedNewExpression((0..0).into()))?;

        use ArithmeticExpression::*;

        match &token.kind {
            TokenKind::Number(value) => Ok(Atom {
                value: *value,
                span: self.advance().span,
            }),

            TokenKind::Operator(op) => match op {
                OperatorKind::Plus => {
                    self.advance();
                    self.factor()
                }
                OperatorKind::Minus => {
                    self.advance();
                    Ok(Unary {
                        operation: UnaryOperation::Negation,
                        expr: Box::new(self.factor()?),
                    })
                }
                _ => Err(ParserError::UnexpectedToken {
                    expected: "a number, a unary operator, or an expression".to_string(),
                    found: token.kind.clone(),
                    span: (usize::max(0, token.span.start - 1)..token.span.end).into(),
                }),
            },

            TokenKind::OpenParenthesis => {
                let start = token.span.start;

                self.advance();
                let expr = self.expression()?;

                if let Some(next) = self.peek() {
                    println!("{:?}", next);

                    if next.kind != TokenKind::CloseParenthesis {
                        let span = (start..next.span.end).into();
                        return Err(ParserError::UnclosedParenthesis(span));
                    }
                }

                self.advance();

                Ok(expr)
            }

            TokenKind::Eof => Err(ParserError::UnexpectedEof(
                (token.span.start - 1..token.span.end).into(),
            )),

            _ => Err(ParserError::UnexpectedToken {
                span: (token.span.start..token.span.end).into(),
                expected: "a number, a unary operator, or an expression".to_string(),
                found: token.kind.clone(),
            }),
        }
    }

    #[allow(dead_code)]
    /// A util function to advance the parser by one token.
    /// If the next token is not of the given kind, an error is returned.
    /// Otherwise, the token is returned.
    fn expect(&mut self, kind: TokenKind, err: String) -> Result<Token> {
        let token = self.tokens.remove(0);

        if token.kind != kind {
            return Err(ParserError::UnexpectedToken {
                expected: err,
                found: token.kind,
                span: token.span,
            });
        }

        Ok(token)
    }

    /// A util function that peeks at the next token.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(0)
    }

    fn advance(&mut self) -> Token {
        self.tokens.remove(0)
    }

    /// A util function that parses a binary operator.
    fn binary_operator(
        &mut self,
        left_fn: fn(&mut Self) -> Result<ArithmeticExpression>,
        ops: &[OperatorKind],
    ) -> Result<ArithmeticExpression> {
        let mut left = left_fn(self)?;

        while let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Operator(operator) if ops.contains(operator) => {}
                _ => break,
            }

            let op = self.tokens.remove(0);
            let right = left_fn(self)?;

            use BinaryOperation::*;
            use OperatorKind::*;
            use TokenKind::*;

            let operation = match op.kind {
                Operator(operator) => match operator {
                    Plus => Addition,
                    Minus => Subtraction,
                    Star => Multiplication,
                    Slash => Division,
                },
                _ => unreachable!(),
            };

            left = ArithmeticExpression::Binary {
                operation,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }

        Ok(left)
    }
}
