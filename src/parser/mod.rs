pub mod ast;
pub mod error;

use crate::{
    lexer::token::{Grouping, Keyword, OpKind, Token},
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

    /// A list of errors produced so far.
    errors: Vec<Spanned<ParsingError>>,
}

impl Parser {
    /// Creates a new [`Parser`].
    pub fn new(tokens: Vec<Spanned<Token>>) -> Self {
        Self {
            tokens,
            cursor: 0,
            errors: Vec::new(),
        }
    }

    /// Parses a full source file.
    pub fn parse_source(&mut self) -> Result<Spanned<Statement>, Vec<Spanned<ParsingError>>> {
        if self.peek() == Some(Token::Eof) {
            return Ok(Spanned::wrap(
                Statement::Program { stmts: vec![] },
                self.consume().unwrap().span,
            ));
        }

        let mut stmts = Vec::new();

        while self.peek() != Some(Token::Eof) {
            let result = self.statement();

            match result {
                Ok(statement) => {
                    // require semicolons to end all (non block) expression statements in a program
                    if let Statement::Expression {
                        expr,
                        has_semicolon,
                    } = &statement.value
                    {
                        let is_block = matches!(expr, Expression::Block { .. });

                        if !has_semicolon && !is_block {
                            if let Err(e) = self.expect(Token::Semicolon, "';'") {
                                self.errors.push(e);
                                self.synchronize();
                            }
                        }
                    }

                    stmts.push(statement);
                }

                // try and synchronize back to a valid start
                Err(error) => {
                    self.errors.push(error);
                    self.synchronize();
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        let span = Span::merge(stmts.first().unwrap().span, stmts.last().unwrap().span);

        Ok(Spanned::wrap(Statement::Program { stmts }, span))
    }

    /// Parses a source file as a REPL file.
    pub fn parse_repl(&mut self) -> StatementResult {
        if self.peek() == Some(Token::Eof) {
            return Ok(Spanned::wrap(
                Statement::Program { stmts: vec![] },
                self.consume()?.span,
            ));
        }

        let mut stmts = Vec::new();
        let mut tail = None;

        while let Some(token) = self.peek()
            && token != Token::Eof
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

        let _ = self.expect(Token::Eof, "the end of file")?;

        let first = stmts
            .first()
            .map(|stmt| stmt.span)
            .unwrap_or_else(|| tail.as_ref().unwrap().span);

        let last = stmts
            .last()
            .map(|stmt| stmt.span)
            .unwrap_or_else(|| tail.as_ref().unwrap().span);

        let span = Span::merge(first, last);

        Ok(Spanned::wrap(Statement::Repl { stmts, tail }, span))
    }

    /// Synchronizes the parser back to a valid starting point after an error.
    fn synchronize(&mut self) {
        while let Some(token) = self.peek() {
            match token {
                // semicolons always delineate the end of a statement, so we should continue from
                // there
                Token::Semicolon
                | Token::Grouping(Grouping::ClosingParen | Grouping::ClosingCurly) => {
                    let _ = self.consume();
                    return;
                }

                // restart at the next statement
                Token::Keyword(
                    Keyword::Let
                    | Keyword::Fn
                    | Keyword::Return
                    | Keyword::While
                    | Keyword::Break
                    | Keyword::Assert,
                ) => {
                    return;
                }

                // stop at EOF
                Token::Eof => return,

                _ => {
                    let _ = self.consume();
                }
            }
        }
    }

    fn statement(&mut self) -> StatementResult {
        match self.peek() {
            Some(Token::Keyword(Keyword::Print)) => self.print(),

            Some(Token::Keyword(Keyword::Let)) => self.let_declaration(),

            Some(Token::Keyword(Keyword::Assert)) => self.assert(),

            Some(Token::Keyword(Keyword::While)) => self.r#while(),

            Some(Token::Keyword(Keyword::Break)) => {
                let statement = self.consume()?.map(|_| Statement::Break);
                self.expect(Token::Semicolon, "';'")?;

                Ok(statement)
            }

            Some(Token::Keyword(Keyword::Return)) => {
                let keyword_span = self.consume()?.span;
                let result = if self.peek() != Some(Token::Semicolon) {
                    Some(self.expr()?)
                } else {
                    None
                };

                let semicolon_span = self.expect(Token::Semicolon, "';'")?.span;
                let span = Span::merge(keyword_span, semicolon_span);

                Ok(Spanned::wrap(Statement::Return { result }, span))
            }

            // function definition statements must have a name associated with them, otherwise we
            // treat them as anonymous function definition
            Some(Token::Keyword(Keyword::Fn))
                if matches!(self.peek_at(1), Some(Token::Symbol(_))) =>
            {
                self.fn_declaration()
            }

            Some(_) => {
                let expr = self.expr()?;

                Ok(expr.map(|expr| Statement::Expression {
                    expr,
                    has_semicolon: match self.peek() {
                        Some(Token::Semicolon) => {
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

    fn print(&mut self) -> StatementResult {
        let keyword_span = self.expect(Token::Keyword(Keyword::Print), "'print'")?.span;
        let expr = self.expr()?;
        let semicolon_span = self.expect(Token::Semicolon, "';'")?.span;

        let span = Span::merge(keyword_span, semicolon_span);

        Ok(Spanned::wrap(Statement::Print(expr), span))
    }

    fn r#while(&mut self) -> StatementResult {
        let keyword_span = self.expect(Token::Keyword(Keyword::While), "'while'")?.span;
        let predicate = self.expr()?;

        let body = self.block()?;

        let mut closing_span = body.span;

        // allow tail semicolons, but don't require them
        if self.peek() == Some(Token::Semicolon) {
            closing_span = self.consume()?.span;
        }

        Ok(Spanned::wrap(
            Statement::While { predicate, body },
            Span::merge(keyword_span, closing_span),
        ))
    }

    fn let_declaration(&mut self) -> StatementResult {
        let keyword_span = self.expect(Token::Keyword(Keyword::Let), "'let'")?.span;

        let token = self.consume()?;
        let Token::Symbol(symbol) = token.value else {
            return Err(token.map(|t| ParsingError::UnexpectedToken {
                expected: "a binding name",
                found: t,
            }));
        };

        self.expect(Token::Operator(OpKind::Assign), "'='")?;

        let expr = self.expr()?;

        let semicolon_span = self.expect(Token::Semicolon, "';'")?.span;

        let span = Span::merge(keyword_span, semicolon_span);

        Ok(Spanned::wrap(
            Statement::Declaration {
                symbol,
                value: expr,
            },
            span,
        ))
    }

    fn fn_declaration(&mut self) -> StatementResult {
        let keyword = self.expect(Token::Keyword(Keyword::Fn), "'fn'")?;

        let token = self.consume()?;
        let Token::Symbol(symbol) = token.value else {
            return Err(token.map(|t| ParsingError::UnexpectedToken {
                expected: "a binding name",
                found: t,
            }));
        };

        let parameters = self.parameters()?.value;

        let body = self.block()?;

        let span = Span::merge(keyword.span, body.span);

        Ok(Spanned::wrap(
            Statement::FunctionDeclaration {
                symbol,
                parameters,
                body,
            },
            span,
        ))
    }

    fn assert(&mut self) -> StatementResult {
        let keyword_span = self
            .expect(Token::Keyword(Keyword::Assert), "'assert'")?
            .span;
        let expr = self.expr()?;
        let semicolon_span = self.expect(Token::Semicolon, "';'")?.span;

        let span = Span::merge(keyword_span, semicolon_span);

        Ok(Spanned::wrap(Statement::Assert(expr), span))
    }

    fn expr(&mut self) -> ExprResult {
        self.assignment()
    }

    fn assignment(&mut self) -> ExprResult {
        let expr = self.or()?;

        // check if this is an assignment expression
        if self.peek() == Some(Token::Operator(OpKind::Assign)) {
            // try to convert the expr into a valid l-value
            let Expression::Variable { symbol } = expr.value else {
                return Err(expr.map(|_| ParsingError::InvalidAssignmentLhs))?;
            };

            let lhs_span = expr.span;

            self.consume()?;

            let value = self.expr()?;

            let span = Span::merge(lhs_span, value.span);

            return Ok(Spanned::wrap(
                Expression::Assignment {
                    symbol: Spanned::wrap(symbol, lhs_span),
                    expr: Box::new(value),
                },
                span,
            ));
        }

        Ok(expr)
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
        if let Some(Token::Operator(op)) = self.peek()
            && let Ok(op) = UnaryOp::try_from(op)
        {
            let op_span = self.consume()?.span;

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
            self.call()
        }
    }

    fn call(&mut self) -> ExprResult {
        let mut expr = self.atom()?;

        while self.peek() == Some(Token::Grouping(Grouping::OpeningParen)) {
            let arguments = self.arguments()?;

            let span = Span::merge(expr.span, arguments.span);

            expr = Spanned::wrap(
                Expression::Call {
                    callee: Box::new(expr),
                    arguments: arguments.value,
                },
                span,
            );
        }

        Ok(expr)
    }

    fn block(&mut self) -> ExprResult {
        let opening = self.expect(Token::Grouping(Grouping::OpeningCurly), "'{'")?;

        let mut stmts = Vec::new();
        let mut tail = None;

        // keep parsing statements until we reach a }
        while let Some(token) = self.peek()
            && !matches!(token, Token::Grouping(Grouping::ClosingCurly) | Token::Eof)
        {
            let result = self.statement();

            match result {
                Ok(statement) => {
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

                Err(error) => {
                    self.errors.push(error);
                    self.synchronize();
                }
            }
        }

        let closing = self.expect(Token::Grouping(Grouping::ClosingCurly), "'}'")?;

        let span = Span::merge(opening.span, closing.span);

        Ok(Spanned::wrap(Expression::Block { stmts, tail }, span))
    }

    fn r#if(&mut self) -> ExprResult {
        let if_token = self.expect(Token::Keyword(Keyword::If), "'if'")?;

        let predicate = Box::new(self.expr()?);

        let body = Box::new(self.block()?);

        // check if we have an else clause
        if self.peek() == Some(Token::Keyword(Keyword::Else)) {
            let else_token = self.consume()?;

            // does the else have another if?
            if self.peek() == Some(Token::Keyword(Keyword::If)) {
                let mut else_clause = self.r#if()?;
                else_clause.span = Span::merge(else_token.span, else_clause.span);

                let span = Span::merge(if_token.span, else_clause.span);

                return Ok(Spanned::wrap(
                    Expression::If {
                        predicate,
                        body,
                        else_clause: Some(Box::new(else_clause)),
                    },
                    span,
                ));
            }

            // parse a normal else body
            let else_body = Box::new(self.block()?);

            let span = Span::merge(if_token.span, else_body.span);

            return Ok(Spanned::wrap(
                Expression::If {
                    predicate,
                    body,
                    else_clause: Some(else_body),
                },
                span,
            ));
        }

        let span = Span::merge(if_token.span, body.span);

        Ok(Spanned::wrap(
            Expression::If {
                predicate,
                body,
                else_clause: None,
            },
            span,
        ))
    }

    /// Parses an atom (simplest part of an expression).
    fn atom(&mut self) -> ExprResult {
        let token = self.consume()?;

        let expression = match token.value {
            Token::Int(int) => Spanned::wrap(Expression::Integer(int), token.span),

            Token::Keyword(Keyword::True) => Spanned::wrap(Expression::Boolean(true), token.span),

            Token::Keyword(Keyword::False) => Spanned::wrap(Expression::Boolean(false), token.span),

            Token::String(string) => Spanned::wrap(Expression::String(string.into()), token.span),

            Token::Symbol(symbol) => Spanned::wrap(Expression::Variable { symbol }, token.span),

            Token::Grouping(Grouping::OpeningParen) => {
                let expr = self.expr()?;
                let next = self.consume()?;

                if next.value != Token::Grouping(Grouping::ClosingParen) {
                    return Err(Spanned::wrap(
                        ParsingError::UnexpectedToken {
                            expected: "')'",
                            found: next.value,
                        },
                        next.span,
                    ));
                }

                Spanned::wrap(expr.value, Span::merge(token.span, next.span))
            }

            Token::Grouping(Grouping::OpeningCurly) => {
                self.rewind();
                return self.block();
            }

            Token::Keyword(Keyword::If) => {
                self.rewind();
                return self.r#if();
            }

            // begin a lambda expression
            Token::Keyword(Keyword::Fn) => {
                let parameters = self.parameters()?.value;
                let body = Box::new(self.expr()?);

                let span = Span::merge(token.span, body.span);

                Spanned::wrap(Expression::Lambda { parameters, body }, span)
            }

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
    fn peek(&mut self) -> Option<Token> {
        self.tokens.get(self.cursor).cloned().map(|s| s.value)
    }

    /// Peeks `n` tokens ahead of the cursor.
    fn peek_at(&mut self, n: usize) -> Option<Token> {
        self.tokens.get(self.cursor + n).cloned().map(|s| s.value)
    }

    /// Rewinds the parser's cursor by one token.
    fn rewind(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Parses a sequence of function parameters.
    fn parameters(&mut self) -> Result<Spanned<Vec<Spanned<&'static str>>>, Spanned<ParsingError>> {
        self.parenthesized_list(|self_| {
            let token = self_.consume()?;

            let Token::Symbol(parameter) = token.value else {
                return Err(Spanned::wrap(
                    ParsingError::UnexpectedToken {
                        expected: "a parameter symbol",
                        found: token.value,
                    },
                    token.span,
                ));
            };

            Ok(token.map(|_| parameter))
        })
    }

    /// Parses a sequence of function arguments.
    fn arguments(&mut self) -> Result<Spanned<Vec<Spanned<Expression>>>, Spanned<ParsingError>> {
        self.parenthesized_list(Self::expr)
    }

    /// Parses a sequence of parseable-items enclosed by '(' and ')' and delimeted by ','.
    fn parenthesized_list<F, T>(
        &mut self,
        mut f: F,
    ) -> Result<Spanned<Vec<T>>, Spanned<ParsingError>>
    where
        F: FnMut(&mut Self) -> Result<T, Spanned<ParsingError>>,
    {
        let opening = self.expect(Token::Grouping(Grouping::OpeningParen), "'('")?;

        let mut result = Vec::new();

        while self.peek() != Some(Token::Grouping(Grouping::ClosingParen)) {
            result.push(f(self)?);

            match self.peek() {
                // advance past the comma
                Some(Token::Comma) => {
                    let _ = self.consume()?;
                }

                Some(Token::Grouping(Grouping::ClosingParen)) => break,

                _ => {
                    return Err(self.consume()?.map(|found| ParsingError::UnexpectedToken {
                        expected: "')' or ','",
                        found,
                    }));
                }
            };
        }

        let closing = self.expect(Token::Grouping(Grouping::ClosingParen), "')'")?;

        Ok(Spanned::wrap(
            result,
            Span::merge(opening.span, closing.span),
        ))
    }

    /// Builds a binary expression by repeatedly applying `f` while the next token matches the
    /// given operators.
    fn reduce_binary_op<F>(&mut self, mut f: F, operators: &[BinaryOp]) -> ExprResult
    where
        F: FnMut(&mut Self) -> ExprResult,
    {
        let mut lhs = f(self)?;

        while let Some(token) = self.peek()
            && let Ok(operator) = BinaryOp::try_from(token)
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

    /// Expects the next consumed token to match the provided token, returning an error if it
    /// didn't.
    fn expect(
        &mut self,
        expected: Token,
        expected_label: &'static str,
    ) -> Result<Spanned<Token>, Spanned<ParsingError>> {
        let token = self.consume()?;

        if token.value != expected {
            return Err(Spanned::wrap(
                match token.value {
                    Token::Eof => ParsingError::UnexpectedEof,
                    _ => ParsingError::UnexpectedToken {
                        expected: expected_label,
                        found: token.value,
                    },
                },
                token.span,
            ));
        }

        Ok(token)
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
        let tokens = Tokenizer::new(make_source(src))
            .collect::<Result<_, _>>()
            .unwrap();

        match Parser::new(tokens).parse_repl().unwrap().value {
            Statement::Repl { mut stmts, tail } => tail
                .map(|expr| expr.value)
                .or_else(|| match stmts.remove(0).value {
                    Statement::Expression { expr, .. } => Some(expr),
                    _ => None,
                })
                .expect("should have at least one expression in repl for testing"),

            other => panic!("expected Expression statement, got {other:?}"),
        }
    }

    fn parse_source(src: &'static str) -> Vec<Spanned<Statement>> {
        let tokens = Tokenizer::new(make_source(src))
            .collect::<Result<_, _>>()
            .unwrap();

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
        let (lhs, op, rhs) = as_binop(parse_repl("1 + 2 * 3"));
        assert_eq!(op, BinaryOp::Plus);
        assert!(matches!(lhs, Expression::Integer(1)));
        let (_, inner_op, _) = as_binop(rhs);
        assert_eq!(inner_op, BinaryOp::Star);
    }

    #[test]
    fn left_associativity() {
        let (lhs, op, rhs) = as_binop(parse_repl("1 - 2 - 3"));
        assert_eq!(op, BinaryOp::Minus);
        let (_, inner_op, _) = as_binop(lhs);
        assert_eq!(inner_op, BinaryOp::Minus);
        assert!(matches!(rhs, Expression::Integer(3)));
    }

    #[test]
    fn logical_and_or_precedence() {
        let (_, op, rhs) = as_binop(parse_repl("true or false and false"));
        assert_eq!(op, BinaryOp::Or);
        let (_, inner_op, _) = as_binop(rhs);
        assert_eq!(inner_op, BinaryOp::And);
    }

    #[test]
    fn parenthesized_overrides_precedence() {
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
        let (_, _, span) = unwrap_expr(parse_source(src).into_iter().next().unwrap());
        assert_eq!(span.text(), "{ 1 + 2 }");
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

    #[test]
    fn span_binop_includes_whitespace_between_operands() {
        let stmts = parse_source("1   +   2;");
        let (expr, _, _) = unwrap_expr(stmts.into_iter().next().unwrap());
        match expr {
            Expression::BinaryOperation { lhs, rhs, .. } => {
                // individual operand spans should not include surrounding whitespace
                assert_eq!(lhs.span.text(), "1");
                assert_eq!(rhs.span.text(), "2");
            }
            other => panic!("{other:?}"),
        }
    }
}
