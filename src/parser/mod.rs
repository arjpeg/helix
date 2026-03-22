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
        if let Some(Spanned {
            value: Token::Eof,
            span,
        }) = self.peek()
        {
            return Ok(Spanned::wrap(Statement::Program { stmts: vec![] }, span));
        }

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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        lexer::Tokenizer,
        parser::{
            Parser,
            ast::{BinaryOp, Expression, Statement, UnaryOp},
        },
        source::{Source, Span, Spanned},
    };

    fn make_source(content: &'static str) -> Source {
        Source {
            content,
            path: Path::new("test.hx"),
        }
    }

    fn parse_repl(src: &'static str) -> Expression {
        let tokens: Vec<_> = Tokenizer::new(make_source(src))
            .map(|t| t.unwrap())
            .collect();

        let mut parser = Parser::new(tokens);
        match parser.parse_repl().unwrap().value {
            Statement::Expression { expr, .. } => expr,
            other => panic!("expected Expression statement, got {other:?}"),
        }
    }
    fn parse_source(src: &'static str) -> Vec<Spanned<Statement>> {
        let tokens: Vec<_> = Tokenizer::new(make_source(src))
            .map(|t| t.unwrap())
            .collect();

        match Parser::new(tokens).parse_source().unwrap().value {
            Statement::Program { stmts } => stmts,
            other => panic!("expected Program, got {other:?}"),
        }
    }

    fn unwrap_expr(stmt: Spanned<Statement>) -> (Expression, bool, Span) {
        match stmt.value {
            Statement::Expression {
                expr,
                has_semicolon,
            } => (expr, has_semicolon, stmt.span),
            other => panic!("expected Expression statement, got {other:?}"),
        }
    }

    fn as_binop(expr: Expression) -> (Expression, BinaryOp, Expression) {
        match expr {
            Expression::BinaryOperation { lhs, operator, rhs } => {
                ((*lhs).value, operator, (*rhs).value)
            }

            other => panic!("expected BinaryOperation, got {other:?}"),
        }
    }

    #[test]
    fn integer_literal() {
        assert!(matches!(parse_repl("42"), Expression::Integer(42)));
    }

    #[test]
    fn boolean_true() {
        assert!(matches!(parse_repl("true"), Expression::Boolean(true)));
    }

    #[test]
    fn boolean_false() {
        assert!(matches!(parse_repl("false"), Expression::Boolean(false)));
    }

    #[test]
    fn unary_negation() {
        let expr = parse_repl("-5");
        match expr {
            Expression::UnaryOperation { operator, operand } => {
                assert_eq!(operator, UnaryOp::Minus);
                assert!(matches!(operand.value, Expression::Integer(5)));
            }
            other => panic!("expected UnaryOperation, got {other:?}"),
        }
    }

    #[test]
    fn unary_bang() {
        let expr = parse_repl("!true");
        match expr {
            Expression::UnaryOperation { operator, operand } => {
                assert_eq!(operator, UnaryOp::Bang);
                assert!(matches!(operand.value, Expression::Boolean(true)));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn addition() {
        let (lhs, op, rhs) = as_binop(parse_repl("1 + 2"));
        assert!(matches!(lhs, Expression::Integer(1)));
        assert_eq!(op, BinaryOp::Plus);
        assert!(matches!(rhs, Expression::Integer(2)));
    }

    #[test]
    fn precedence_mul_over_add() {
        // 1 + 2 * 3  =>  1 + (2 * 3)
        let (lhs, op, rhs) = as_binop(parse_repl("1 + 2 * 3"));
        assert_eq!(op, BinaryOp::Plus);
        assert!(matches!(lhs, Expression::Integer(1)));
        let (_, inner_op, _) = as_binop(rhs);
        assert_eq!(inner_op, BinaryOp::Star);
    }

    #[test]
    fn left_associativity() {
        // 1 - 2 - 3  =>  (1 - 2) - 3
        let (lhs, op, rhs) = as_binop(parse_repl("1 - 2 - 3"));
        assert_eq!(op, BinaryOp::Minus);
        let (_, inner_op, _) = as_binop(lhs);
        assert_eq!(inner_op, BinaryOp::Minus);
        assert!(matches!(rhs, Expression::Integer(3)));
    }

    #[test]
    fn logical_and_or_precedence() {
        // true or false and false  =>  true or (false and false)
        let (_, op, rhs) = as_binop(parse_repl("true or false and false"));
        assert_eq!(op, BinaryOp::Or);
        let (_, inner_op, _) = as_binop(rhs);
        assert_eq!(inner_op, BinaryOp::And);
    }

    #[test]
    fn parenthesized_overrides_precedence() {
        // (1 + 2) * 3  =>  (1 + 2) * 3, outer op is *
        let (lhs, op, _) = as_binop(parse_repl("(1 + 2) * 3"));
        assert_eq!(op, BinaryOp::Star);
        let (_, inner_op, _) = as_binop(lhs);
        assert_eq!(inner_op, BinaryOp::Plus);
    }

    #[test]
    fn empty_block() {
        match parse_repl("{}") {
            Expression::Block { stmts, tail } => {
                assert!(stmts.is_empty());
                assert!(tail.is_none());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn block_with_tail() {
        match parse_repl("{ 1 + 2 }") {
            Expression::Block { stmts, tail } => {
                assert!(stmts.is_empty());
                let tail_expr = tail.expect("expected tail").value;
                let (_, op, _) = as_binop(tail_expr);
                assert_eq!(op, BinaryOp::Plus);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn block_with_stmt_and_tail() {
        match parse_repl("{ 1 + 1; 99 }") {
            Expression::Block { stmts, tail } => {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(tail.unwrap().value, Expression::Integer(99)));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unclosed_paren_is_error() {
        let tokens: Vec<_> = Tokenizer::new(make_source("(1 + 2"))
            .map(|t| t.unwrap())
            .collect();
        assert!(Parser::new(tokens).parse_repl().is_err());
    }

    #[test]
    fn missing_semicolon_in_source_is_error() {
        let tokens: Vec<_> = Tokenizer::new(make_source("1 + 2\n3 + 4"))
            .map(|t| t.unwrap())
            .collect();
        assert!(Parser::new(tokens).parse_source().is_err());
    }

    #[test]
    fn program_single_statement() {
        let stmts = parse_source("1 + 2;");
        assert_eq!(stmts.len(), 1);
        let (expr, has_semi, _) = unwrap_expr(stmts.into_iter().next().unwrap());
        assert!(has_semi);
        assert!(matches!(
            expr,
            Expression::BinaryOperation {
                operator: BinaryOp::Plus,
                ..
            }
        ));
    }

    #[test]
    fn program_multiple_statements() {
        let stmts = parse_source("1 + 2;\n3 * 4;\ntrue;");
        assert_eq!(stmts.len(), 3);
    }

    #[test]
    fn program_mixed_expr_types() {
        let stmts = parse_source("true;\n-5;\n1 == 1;");
        assert_eq!(stmts.len(), 3);

        assert!(matches!(
            unwrap_expr(stmts[0].clone()).0,
            Expression::Boolean(true)
        ));
        assert!(matches!(
            unwrap_expr(stmts[1].clone()).0,
            Expression::UnaryOperation {
                operator: UnaryOp::Minus,
                ..
            }
        ));
        assert!(matches!(
            unwrap_expr(stmts[2].clone()).0,
            Expression::BinaryOperation {
                operator: BinaryOp::Equals,
                ..
            }
        ));
    }

    #[test]
    fn program_block_statement_no_semicolon_required() {
        // blocks don't need a trailing semicolon
        let stmts = parse_source("{ 1; 2 }");
        assert_eq!(stmts.len(), 1);
        assert!(matches!(
            unwrap_expr(stmts[0].clone()).0,
            Expression::Block { .. }
        ));
    }

    #[test]
    fn program_missing_semicolon_is_error() {
        let tokens: Vec<_> = Tokenizer::new(make_source("1 + 2\n3 + 4;"))
            .map(|t| t.unwrap())
            .collect();
        assert!(Parser::new(tokens).parse_source().is_err());
    }

    #[test]
    fn program_empty() {
        let stmts = parse_source("");
        assert_eq!(stmts.len(), 0);
    }

    #[test]
    fn span_integer_literal() {
        let stmts = parse_source("123;");
        let (_, _, span) = unwrap_expr(stmts.into_iter().next().unwrap());
        assert_eq!(span.text(), "123");
    }

    #[test]
    fn span_binary_op_covers_both_operands() {
        // span of `1 + 2` should cover the whole expression
        let stmts = parse_source("1 + 2;");
        let (expr, _, _) = unwrap_expr(stmts.into_iter().next().unwrap());
        match expr {
            Expression::BinaryOperation { lhs, rhs, .. } => {
                assert_eq!(lhs.span.text(), "1");
                assert_eq!(rhs.span.text(), "2");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn span_unary_covers_operator_and_operand() {
        let stmts = parse_source("-42;");
        let (expr, _, _) = unwrap_expr(stmts.into_iter().next().unwrap());
        match expr {
            Expression::UnaryOperation { operand, .. } => {
                assert_eq!(operand.span.text(), "42");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn span_block_covers_braces() {
        let src = "{ 1 + 2 }";
        let tokens: Vec<_> = Tokenizer::new(make_source(src))
            .map(|t| t.unwrap())
            .collect();

        let _ = match Parser::new(tokens).parse_repl().unwrap().value {
            Statement::Expression { expr, .. } => expr,
            other => panic!("{other:?}"),
        };

        let tokens: Vec<_> = Tokenizer::new(make_source(src))
            .map(|t| t.unwrap())
            .collect();
        let spanned = Parser::new(tokens).parse_repl().unwrap();
        assert_eq!(spanned.span.text(), "{ 1 + 2 }");
    }

    #[test]
    fn span_nested_binop_lhs_rhs_independent() {
        // `1 + 2 * 3`: lhs span = "1", rhs span = "2 * 3"
        let stmts = parse_source("1 + 2 * 3;");
        let (expr, _, _) = unwrap_expr(stmts.into_iter().next().unwrap());
        match expr {
            Expression::BinaryOperation { lhs, rhs, .. } => {
                assert_eq!(lhs.span.text(), "1");
                assert_eq!(rhs.span.text(), "2 * 3");
            }
            other => panic!("{other:?}"),
        }
    }
}
