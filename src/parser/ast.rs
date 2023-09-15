use std::fmt::Display;

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

    /// A string literal, such as `"hello"`
    StringLiteral(String),

    /// A variable reference, such as `x`
    VariableReference(String),

    /// No operation
    NoOp,

    /// An assignment expression, such as `x = 1`
    Assignment {
        /// The name of the variable being assigned to
        name: String,
        /// The value being assigned
        value: Box<AstNode>,
        /// If the assignment includes a declaration
        /// (e.g. `let x = 1`)
        declaration: bool,
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

    /// A print statement
    Print { expression: Box<AstNode> },

    /// A while loop
    While {
        condition: Box<AstNode>,
        body: Box<AstNode>,
    },

    /// A function statement
    /// (Not a function expression)
    FunctionDefinition {
        params: Vec<String>,
        body: Box<AstNode>,
        name: String,
    },
}

impl Display for AstNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AstNodeKind as Kind;

        write!(
            f,
            "{}",
            match self {
                Kind::BinaryExpression { .. } => "Binary Expression",
                Kind::UnaryExpression { .. } => "Unary Expression",
                Kind::Block { .. } => "Block",
                Kind::NumberLiteral(_) => "Number Literal",
                Kind::StringLiteral(_) => "String Literal",
                Kind::VariableReference(_) => "Variable Reference",
                Kind::NoOp => "NoOp",
                Kind::Assignment { .. } => "Assignment",
                Kind::If { .. } => "If Statement",
                Kind::Else { .. } => "Else Statement",
                Kind::Print { .. } => "Print Statement",
                Kind::While { .. } => "While Loop",
                Kind::FunctionDefinition { .. } => "Function Definition",
            }
        )
    }
}
