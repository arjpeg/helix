pub mod error;
pub mod value;

use crate::{
    interpreter::{error::RuntimeError, value::Value},
    parser::ast::{Expression, Statement},
    source::{Span, Spanned},
};

type Result<T> = std::result::Result<T, Spanned<RuntimeError>>;

/// A basic tree walking interpreter, responsible for evaluating source ASTs.
#[derive(Debug, Clone)]
pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    /// Excecutes a source file, running it until completion.
    pub fn excecute(&mut self, tree: &Spanned<Statement>) -> Result<Option<Value>> {
        self.statement(&tree.value, tree.span)
    }

    fn statement(&mut self, statement: &Statement, span: Span) -> Result<Option<Value>> {
        match statement {
            Statement::Program { stmts } => {
                for Spanned {
                    value: statement,
                    span,
                } in stmts
                {
                    self.statement(statement, *span)?;
                }
            }

            Statement::Expression { expr, .. } => {
                return Ok(Some(self.expression(expr, span)?.value));
            }
        };

        Ok(None)
    }

    fn expression(&mut self, expression: &Expression, span: Span) -> Result<Spanned<Value>> {
        match expression {
            Expression::Integer(n) => Ok(Spanned::wrap(Value::Integer(*n), span)),

            Expression::Boolean(b) => Ok(Spanned::wrap(Value::Boolean(*b), span)),

            Expression::BinaryOperation { lhs, operator, rhs } => {
                let lhs_result = self.expression(&lhs.value, lhs.span)?;
                let rhs_result = self.expression(&rhs.value, rhs.span)?;

                Value::binary_operation(lhs_result.value, *operator, rhs_result.value)
                    .map(|value| Spanned::wrap(value, span))
                    .map_err(|error| Spanned::wrap(error, span))
            }

            Expression::UnaryOperation { operator, operand } => {
                let operand = self.expression(&operand.value, operand.span)?.value;

                Value::unary_operation(*operator, operand)
                    .map(|value| Spanned::wrap(value, span))
                    .map_err(|error| Spanned::wrap(error, span))
            }

            Expression::Block { stmts, tail } => {
                for Spanned {
                    value: statement,
                    span,
                } in stmts
                {
                    self.statement(statement, *span)?;
                }

                if let Some(tail) = tail {
                    self.expression(&tail.value, tail.span)
                } else {
                    Ok(Spanned::wrap(Value::Unit, span))
                }
            }
        }
    }
}
