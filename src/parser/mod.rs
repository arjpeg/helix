pub mod ast;

use crate::lexer::{
    span::Span,
    token::{OperatorKind, Token, TokenKind},
};

use self::ast::{AstNode, AstNodeKind};

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
    pub fn parse(&mut self) -> AstNode {
        match self.tokens.len() {
            0 => AstNode {
                kind: AstNodeKind::Empty,
                span: Span::new(0, 0),
            },
            _ => self.parse_expr(),
        }
    }

    /// Parses an expression. (TERM) (PLUS|MINUS TERM)*
    fn parse_expr(&mut self) -> AstNode {
        self.parse_binary_expr(
            Self::parse_term,
            &[OperatorKind::Plus, OperatorKind::Minus],
            None::<fn(&mut Self) -> AstNode>,
        )
    }

    /// Parses a term. (FACTOR) (MULT|DIV FACTOR)*
    fn parse_term(&mut self) -> AstNode {
        self.parse_binary_expr(
            Self::parse_factor,
            &[OperatorKind::Star, OperatorKind::Slash],
            None::<fn(&mut Self) -> AstNode>,
        )
    }

    /// Parses a factor. (NUMBER) | (LPAREN EXPR RPAREN)
    fn parse_factor(&mut self) -> AstNode {
        let token = self.peek().unwrap().clone();

        match token.token_kind {
            TokenKind::Number(num) => {
                self.advance();

                AstNode {
                    kind: AstNodeKind::NumberLiteral(num),
                    span: token.span,
                }
            }
            _ => panic!("Expected number or parenthesis"),
        }
    }

    /// Parses a binary expression.
    /// Starts with a left hand side, then parses the operator
    /// (as long as it exists), then parses the right hand side.
    fn parse_binary_expr(
        &mut self,
        left_fn: impl Fn(&mut Self) -> AstNode,
        operators: &[OperatorKind],
        right_fn: Option<impl Fn(&mut Self) -> AstNode>,
    ) -> AstNode {
        // If right is none, then we just use left_fn
        let mut lhs = left_fn(self);
        let start = lhs.span.start;

        // FIXME: I am pretty sure this is wrong
        while let Some(op) = self.clone().peek() {
            dbg!(op);
            if !matches!(op.token_kind, TokenKind::Operator(op) if operators.contains(&op)) {
                break;
            }

            self.advance();

            let rhs = match right_fn.as_ref() {
                Some(right_fn) => right_fn(self),
                None => left_fn(self),
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

        lhs
    }

    /// Moves forward in the list of tokens.
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Peeks at the current token.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
}
