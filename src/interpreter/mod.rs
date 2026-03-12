pub mod value;

use crate::{
    interpreter::value::Value,
    parser::ast::{Expression, Statement},
    source::{Span, Spanned},
};

/// A basic tree walking interpreter, responsible for evaluating source ASTs.
#[derive(Debug, Clone)]
pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    /// Excecutes a source file, running it until completion.
    pub fn excecute(&mut self, tree: &Spanned<Statement>) {
        self.statement(&tree.value, tree.span).unwrap();
    }

    fn statement(&mut self, statement: &Statement, span: Span) -> Result<(), ()> {
        match statement {
            Statement::Expression { expr } => dbg!(self.expression(expr, span)),
        }?;

        Ok(())
    }

    fn expression(&mut self, expression: &Expression, span: Span) -> Result<Spanned<Value>, ()> {
        match expression {
            Expression::Integer(n) => Ok(Spanned::wrap(Value::Integer(*n), span)),

            Expression::BinaryOperation { lhs, operator, rhs } => {
                let lhs_result = self.expression(&lhs.value, lhs.span)?;
                let rhs_result = self.expression(&rhs.value, rhs.span)?;

                Value::binary_operation(lhs_result.value, *operator, rhs_result.value)
                    .map(|value| Spanned::wrap(value, span))
            }

            Expression::UnaryOperation { operator, operand } => {
                let operand = self.expression(&operand.value, operand.span)?.value;

                Value::unary_operation(*operator, operand).map(|value| Spanned::wrap(value, span))
            }
        }
    }
}
