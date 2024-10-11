use crate::token::{Operator, Span, UnaryOperator};

/// A node in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// The kind of the node.
    pub kind: NodeKind,
    /// The span of the node.
    pub span: Span,
}

/// The kind of a node in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
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
        /// The unary operator.
        operator: UnaryOperator,
        /// The operand.
        operand: Box<Node>,
    },

    /// An integer literal.
    Integer(i64),

    /// A floating point literal.
    Float(f64),

    /// A boolean literal.
    Boolean(bool),

    /// A reference to an identifier
    Identifier(String),
}

impl Node {
    /// Create a new node with the given kind and span.
    pub fn new(kind: NodeKind, span: Span) -> Self {
        Self { kind, span }
    }
}
