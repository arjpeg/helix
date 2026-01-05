pub mod ast;
pub mod error;

use crate::{
    lexer::token::{OpKind, Token},
    parser::{
        ast::{Expression, Statement},
        error::{ParsingError, Result},
    },
    source::{Span, Spanned},
};

type StatementResult = Result<Spanned<Statement>>;
type ExprResult = Result<Spanned<Expression>>;

/// Converts a list of [`Token`]s into an Abstract Syntax Tree using recursive descent.
pub struct Parser {
    /// The list of tokens to parse.
    tokens: Vec<Spanned<Token>>,
    /// The current index of token to be parsed.
    cursor: usize,
}

impl Parser {
    /// Creates a new [`Parser`].
    pub fn new(tokens: Vec<Spanned<Token>>) -> Self {
        Self { tokens, cursor: 0 }
    }

    /// Parses a full source file.
    pub fn parse_source(&mut self) -> StatementResult {
        let expr = self.expr()?;

        if let Some(token) = self.peek() {
            return Err(token.map(|t| ParsingError::UnexpectedToken {
                expected: "the end of file",
                found: t,
            }));
        }

        Ok(expr.map(|expr| Statement::Expression { expr }))
    }

    fn expr(&mut self) -> ExprResult {
        self.binary_expr()
    }

    fn binary_expr(&mut self) -> ExprResult {
        self.reduce_binary_op(Self::term, &[OpKind::Plus, OpKind::Minus])
    }

    fn term(&mut self) -> ExprResult {
        self.reduce_binary_op(Self::atom, &[OpKind::Star, OpKind::Slash])
    }

    /// Parses an atom (simplest part of an expression).
    fn atom(&mut self) -> ExprResult {
        let token = self.consume()?;

        let expression = match token.value {
            Token::Integer(int) => Expression::Integer(int),
            found => {
                return Err(Spanned::wrap(
                    ParsingError::UnexpectedToken {
                        expected: "an atom",
                        found: found,
                    },
                    token.span,
                ));
            }
        };

        Ok(Spanned::wrap(expression, token.span))
    }

    /// Consumes a single token, returning an error if it wasn't present.
    fn consume(&mut self) -> Result<Spanned<Token>> {
        let result = self.tokens.get(self.cursor).cloned().ok_or(Spanned::wrap(
            ParsingError::UnexpectedEof,
            self.tokens.last().unwrap().span,
        ));

        self.cursor += 1;

        result
    }

    /// Peeks at the next token without advancing the cursor.
    fn peek(&mut self) -> Option<Spanned<Token>> {
        self.tokens.get(self.cursor).cloned()
    }

    /// Builds a binary expression by repeatedly applying `f` while the next token matches the
    /// given operators.
    fn reduce_binary_op<F>(&mut self, mut f: F, ops: &[OpKind]) -> ExprResult
    where
        F: FnMut(&mut Self) -> ExprResult,
    {
        let mut lhs = f(self)?;

        while let Some(token) = self.peek()
            && let Token::Operator(op) = token.value
            && ops.contains(&op)
        {
            self.consume()?;

            let rhs = f(self)?;
            let span = Span::merge(lhs.span, rhs.span);

            lhs = Spanned::wrap(
                Expression::BinaryOperation {
                    lhs: Box::new(lhs),
                    operator: op,
                    rhs: Box::new(rhs),
                },
                span,
            );
        }

        Ok(lhs)
    }
}
