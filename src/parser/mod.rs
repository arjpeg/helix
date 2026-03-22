pub mod ast;
pub mod error;

use crate::{
    lexer::token::{Grouping, Keyword, Token},
    parser::{
        ast::{BinaryOp, Expression, Statement, UnaryOp},
        error::ParsingError,
    },
    source::{Span, Spanned},
};

type StatementResult = Result<Spanned<Statement>, Spanned<ParsingError>>;
type ExprResult = Result<Spanned<Expression>, Spanned<ParsingError>>;

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
        let mut stmts = Vec::new();

        while self.peek().map(|t| t.value) != Some(Token::Eof) {
            let statement = self.statement()?;

            // require semicolons to end all (non block) expression statements in a program
            if let Statement::Expression {
                expr,
                has_semicolon,
            } = &statement.value
            {
                let is_block = matches!(expr, Expression::Block { .. });

                if !has_semicolon && !is_block {
                    return Err(Spanned::wrap(
                        ParsingError::UnexpectedToken {
                            expected: "a ';'",
                            found: self.peek().unwrap().value,
                        },
                        statement.span,
                    ));
                }
            }

            stmts.push(statement);
        }

        let span = Span::merge(stmts.first().unwrap().span, stmts.last().unwrap().span);

        Ok(Spanned::wrap(Statement::Program { stmts }, span))
    }

    /// Parses a source file as a REPL file.
    pub fn parse_repl(&mut self) -> StatementResult {
        // quick fix for empty repl inputs
        if let Some(Spanned {
            value: Token::Eof,
            span,
        }) = self.peek()
        {
            return Ok(Spanned::wrap(Statement::Program { stmts: vec![] }, span));
        }

        let expr = self.expr()?;

        match self.peek() {
            Some(Spanned {
                value: Token::Eof | Token::Semicolon,
                ..
            }) => Ok(expr.map(|expr| Statement::Expression {
                expr,
                has_semicolon: false,
            })),

            Some(token) => Err(token.map(|t| ParsingError::UnexpectedToken {
                expected: "the end of file",
                found: t,
            })),

            _ => unreachable!("should always have an EOF token"),
        }
    }

    fn statement(&mut self) -> StatementResult {
        match self.peek() {
            Some(_) => {
                let expr = self.expr()?;

                Ok(expr.map(|expr| Statement::Expression {
                    expr,
                    has_semicolon: match self.peek() {
                        Some(Spanned {
                            value: Token::Semicolon,
                            ..
                        }) => {
                            let _ = self.consume();
                            true
                        }

                        _ => false,
                    },
                }))
            }

            _ => unreachable!("should always have an EOF token"),
        }
    }

    fn expr(&mut self) -> ExprResult {
        self.or()
    }

    fn or(&mut self) -> ExprResult {
        self.reduce_binary_op(Self::and, &[BinaryOp::Or])
    }

    fn and(&mut self) -> ExprResult {
        self.reduce_binary_op(Self::equality, &[BinaryOp::And])
    }

    fn equality(&mut self) -> ExprResult {
        self.reduce_binary_op(Self::comparison, &[BinaryOp::Equals, BinaryOp::NotEquals])
    }

    fn comparison(&mut self) -> ExprResult {
        self.reduce_binary_op(
            Self::term,
            &[
                BinaryOp::GreaterThan,
                BinaryOp::GreaterThanEquals,
                BinaryOp::LessThan,
                BinaryOp::LessThanEquals,
            ],
        )
    }

    fn term(&mut self) -> ExprResult {
        self.reduce_binary_op(Self::factor, &[BinaryOp::Plus, BinaryOp::Minus])
    }

    fn factor(&mut self) -> ExprResult {
        self.reduce_binary_op(Self::unary, &[BinaryOp::Star, BinaryOp::Slash])
    }

    fn unary(&mut self) -> ExprResult {
        if let Some(Spanned {
            value: Token::Operator(op),
            span: op_span,
        }) = self.peek()
            && let Ok(op) = UnaryOp::try_from(op)
        {
            self.consume()?;

            let expression = self.unary()?;
            let span = Span::merge(op_span, expression.span);

            Ok(Spanned::wrap(
                Expression::UnaryOperation {
                    operator: op,
                    operand: Box::new(expression),
                },
                span,
            ))
        } else {
            self.atom()
        }
    }

    fn block(&mut self, opening: Spanned<Token>) -> ExprResult {
        let Token::Grouping(Grouping::OpeningCurly) = opening.value else {
            return Err(opening.map(|t| ParsingError::UnexpectedToken {
                expected: "to find find '{'",
                found: t,
            }));
        };

        let mut stmts = Vec::new();
        let mut tail = None;

        // keep parsing statements until we reach a }
        while let Some(token) = self.peek()
            && !matches!(
                token.value,
                Token::Grouping(Grouping::ClosingCurly) | Token::Eof,
            )
        {
            let statement = self.statement()?;

            if let Statement::Expression {
                expr,
                has_semicolon: false,
            } = statement.value
            {
                tail = Some(Box::new(Spanned::wrap(expr, statement.span)));
                break;
            }

            stmts.push(statement);
        }

        let closing = self.consume()?;

        let Token::Grouping(Grouping::ClosingCurly) = closing.value else {
            return Err(closing.map(|t| ParsingError::UnexpectedToken {
                expected: "to find find '}'",
                found: t,
            }));
        };

        let span = Span::merge(opening.span, closing.span);

        Ok(Spanned::wrap(Expression::Block { stmts, tail }, span))
    }

    /// Parses an atom (simplest part of an expression).
    fn atom(&mut self) -> ExprResult {
        let token = self.consume()?;

        let expression = match token.value {
            Token::Int(int) => Spanned::wrap(Expression::Integer(int), token.span),

            Token::Keyword(Keyword::True) => Spanned::wrap(Expression::Boolean(true), token.span),

            Token::Keyword(Keyword::False) => Spanned::wrap(Expression::Boolean(false), token.span),

            Token::Grouping(Grouping::OpeningParen) => {
                let expr = self.expr()?;
                let next = self.consume()?;

                if next.value != Token::Grouping(Grouping::ClosingParen) {
                    return Err(Spanned::wrap(
                        ParsingError::UnexpectedToken {
                            expected: "to find a closing parenthesis",
                            found: next.value,
                        },
                        next.span,
                    ));
                }

                Spanned::wrap(expr.value, Span::merge(token.span, next.span))
            }

            Token::Grouping(Grouping::OpeningCurly) => return self.block(token),

            found => {
                return Err(Spanned::wrap(
                    ParsingError::UnexpectedToken {
                        expected: "an expression",
                        found,
                    },
                    token.span,
                ));
            }
        };

        Ok(expression)
    }

    /// Consumes a single token, returning an error if it wasn't present.
    fn consume(&mut self) -> Result<Spanned<Token>, Spanned<ParsingError>> {
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
    fn reduce_binary_op<F>(&mut self, mut f: F, operators: &[BinaryOp]) -> ExprResult
    where
        F: FnMut(&mut Self) -> ExprResult,
    {
        let mut lhs = f(self)?;

        while let Some(token) = self.peek()
            && let Ok(operator) = BinaryOp::try_from(token.value)
            && operators.contains(&operator)
        {
            self.consume()?;

            let rhs = f(self)?;
            let span = Span::merge(lhs.span, rhs.span);

            lhs = Spanned::wrap(
                Expression::BinaryOperation {
                    lhs: Box::new(lhs),
                    operator,
                    rhs: Box::new(rhs),
                },
                span,
            );
        }

        Ok(lhs)
    }
}
