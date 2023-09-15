use std::rc::Rc;

use crate::lexer::{span::Span, token::Token};

use super::ast::AstNode;

/// An error that occurred during parsing.
#[derive(Debug, Clone)]
pub enum ParserError {
    /// An unexpected token was found.
    UnexpectedToken {
        /// The token that was found.
        found: Token,
        /// The expected token.
        expected: String,
    },

    /// An unexpected end of input was found.
    UnexpectedEof {
        /// The expected token.
        expected: String,
        file: Rc<str>,
    },

    UnexpectedNewline {
        expected: String,
        span: Span,
    },

    /// An unmatched closing parenthesis was found.
    UnmatchedClosingParen {
        /// The closing parenthesis.
        paren: Token,
    },

    /// The left hand side of an assignment was not a valid target.
    InvalidAssignmentTarget {
        /// The expression that was found.
        found: AstNode
    },
}
