use crate::tokens::Span;

/// Binary Operation.
#[derive(Debug, PartialEq)]
pub enum BinaryOperation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

/// Unary Operation.
#[derive(Debug, PartialEq)]
pub enum UnaryOperation {
    Negation,
}

/// An Arithmetic Expression.
#[derive(Debug, PartialEq)]
pub enum ArithmeticExpression {
    Binary {
        operation: BinaryOperation,
        lhs: Box<ArithmeticExpression>,
        rhs: Box<ArithmeticExpression>,
    },

    Unary {
        operation: UnaryOperation,
        expr: Box<ArithmeticExpression>,
    },

    Atom {
        value: f64,
        span: Span,
    },
}

#[derive(Debug, PartialEq)]
pub enum ParseTree {
    Expression(ArithmeticExpression),
    Empty,
}
