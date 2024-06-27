use crate::token::{Operator, UnaryOperator};

/// A node in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A binary operation.
    BinaryOp {
        /// The left hand side of the operation.
        lhs: Box<Node>,
        /// The operator.
        operator: Operator,
        /// The right hand side of the operation.
        rhs: Box<Node>,
    },

    /// A unary operation.
    UnaryOp {
        /// The operator.
        operator: UnaryOperator,
        /// The operand.
        operand: Box<Node>,
    },

    /// An integer literal.
    Integer(i64),

    /// A floating point literal.
    Float(f64),
}
