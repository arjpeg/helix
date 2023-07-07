use crate::lexer::span::Span;
use crate::lexer::token::OperatorKind;

/// A node in the AST
#[derive(Debug, Clone)]
pub struct AstNode {
    /// The kind of node
    pub kind: AstNodeKind,
    /// The location of the node in the source code
    pub span: Span,
}

/// All the different kinds of nodes in the AST
#[derive(Debug, Clone)]
pub enum AstNodeKind {
    /// A binary expression, such as `1 + 1`
    BinaryExpression {
        /// The left hand side of the expression
        lhs: Box<AstNode>,
        /// The operator of the expression
        op: OperatorKind,
        /// The right hand side of the expression
        rhs: Box<AstNode>,
    },

    /// A unary expression, such as `-1`
    UnaryExpression {
        op: OperatorKind,
        expr: Box<AstNode>,
    },

    /// A block expression, such as `{ 1 + 1 }`
    Block {
        /// The expressions inside the block
        expressions: Vec<AstNode>,
    },

    /// A number literal, such as `1`
    NumberLiteral(f64),

    /// A variable reference, such as `x`
    VariableReference(String),

    /// An empty tree
    Empty,

    /// An assignment expression, such as `x = 1`
    Assignment {
        /// The name of the variable being assigned to
        name: String,
        /// The value being assigned
        value: Box<AstNode>,
    },

    /// An if statement. The `else` branch is optional
    If {
        /// The condition of the if expression
        condition: Box<AstNode>,
        /// The body of the if expression
        body: Box<AstNode>,
        /// The else branch of the if expression
        else_branch: Option<Box<AstNode>>,
    },

    /// An else statement
    Else {
        /// The body of the else expression
        body: Box<AstNode>,
    },
}
