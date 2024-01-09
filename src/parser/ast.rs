use std::fmt::Display;

use crate::lexer::span::Span;
use crate::lexer::token::OperatorKind;

/// A node in the AST
#[derive(Debug, Clone, PartialEq)]
pub struct AstNode {
    /// The kind of node
    pub kind: AstNodeKind,
    /// The location of the node in the source code
    pub span: Span,
}

/// All the different kinds of nodes in the AST
#[derive(Debug, Clone, PartialEq)]
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

    /// A continue statement
    Continue,

    /// A break statement
    Break,

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
        write!(
            f,
            "{}",
            match self {
                Self::BinaryExpression { .. } => "Binary Expression",
                Self::UnaryExpression { .. } => "Unary Expression",
                Self::Block { .. } => "Block",
                Self::NumberLiteral(_) => "Number Literal",
                Self::StringLiteral(_) => "String Literal",
                Self::VariableReference(_) => "Variable Reference",
                Self::NoOp => "NoOp",
                Self::Assignment { .. } => "Assignment",
                Self::If { .. } => "If Statement",
                Self::Else { .. } => "Else Statement",
                Self::Print { .. } => "Print Statement",
                Self::While { .. } => "While Loop",
                Self::Continue => "Continue Statement",
                Self::Break => "Break Statement",
                Self::FunctionDefinition { .. } => "Function Definition",
            }
        )
    }
}
