use crate::{
    ast::*,
    error::Result,
    token::{ASTNode, Operator, UnaryOperator},
    value::{Value, ValueKind},
};

use NodeKind as NK;

/// Excecutes a source file, and holds information about the current excecution context.
#[derive(Debug)]
pub struct Interpreter {}

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new() -> Self {
        Self {}
    }

    /// Starts running the interpreter on the given AST.
    pub fn run(&mut self, ast: ASTNode) -> Result<Value> {
        self.visit(ast)
    }

    fn visit(&mut self, node: ASTNode) -> Result<Value> {
        match node.kind {
            NK::Integer(_) | NK::Float(_) | NK::Boolean(_) => Ok(self.construct_literal(node)),

            NK::BinaryOp { lhs, operator, rhs } => self.visit_binary_op(*lhs, operator, *rhs),
            NK::UnaryOp { operator, operand } => self.visit_unary_op(operator, *operand),
            NK::Identifier(_) => todo!(),
        }
    }

    fn visit_binary_op(&mut self, lhs: ASTNode, op: Operator, rhs: ASTNode) -> Result<Value> {
        let lhs = self.visit(lhs)?;
        let rhs = self.visit(rhs)?;

        match op {
            Operator::Plus => lhs.add(&rhs),
            _ => todo!(),
        }
    }

    fn visit_unary_op(&mut self, operator: UnaryOperator, operand: ASTNode) -> Result<Value> {
        let operand = self.visit(operand)?;

        match operator {
            UnaryOperator::Not => operand.not(),
            UnaryOperator::Minus => operand.negate(),
            UnaryOperator::Plus => Ok(operand),
        }
    }

    fn construct_literal(&mut self, node: ASTNode) -> Value {
        let value = match node.kind {
            NK::Integer(value) => ValueKind::Integer(value),
            NK::Float(value) => ValueKind::Float(value),
            NK::Boolean(value) => ValueKind::Boolean(value),
            _ => panic!("visit_literal was called on a non literal ast node, {node:?}"),
        };

        Value {
            kind: value,
            span: node.span,
        }
    }
}
