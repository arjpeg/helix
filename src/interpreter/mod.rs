mod data;
pub mod error;

use std::collections::HashMap;

use crate::{
    lexer::token::OperatorKind,
    parser::ast::{AstNode, AstNodeKind},
};

use self::{
    data::{Value, ValueKind},
    error::InterpreterError,
};

/// A struct that represents an interpreter.
pub struct Interpreter {
    /// The AST that the interpreter will interpret.
    pub ast: Option<AstNode>,

    /// The variables in context.
    pub variables: HashMap<String, Value>,
}

// A type alias for the result of the interpreter.
type InterpreterResult<T> = Result<T, InterpreterError>;

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new(ast: Option<AstNode>) -> Self {
        Self {
            ast,
            variables: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn start(mut self) -> InterpreterResult<Value> {
        self.interpret(
            self.ast
                .clone()
                .expect("can't start interpreter without ast"),
        )
    }

    /// Interprets the AST.
    pub fn interpret(&mut self, ast: AstNode) -> InterpreterResult<Value> {
        match ast.kind {
            AstNodeKind::NumberLiteral(number) => Ok(Value {
                kind: ValueKind::Number(number),
                span: ast.span,
            }),

            AstNodeKind::BinaryExpression { lhs, op, rhs } => {
                Ok(self.interpret_binary_expr(*lhs, op, *rhs)?)
            }

            AstNodeKind::Assignment { name, value } => {
                let value = self.interpret(*value)?;
                self.variables.insert(name, value.clone());

                Ok(value)
            }

            AstNodeKind::VariableReference(name) => {
                Ok(self.variables.get(&name).cloned().ok_or_else(|| {
                    InterpreterError::UndefinedVariable {
                        name,
                        span: ast.span,
                    }
                })?)
            }

            _ => todo!(),
        }
    }

    /// Interprets a binary expression.
    fn interpret_binary_expr(
        &mut self,
        lhs: AstNode,
        op: OperatorKind,
        rhs: AstNode,
    ) -> InterpreterResult<Value> {
        let lhs_value = self.interpret(lhs)?;
        let rhs_value = self.interpret(rhs)?;

        use OperatorKind::*;

        match op {
            Plus => lhs_value.add(&rhs_value),
            Minus => lhs_value.subtract(&rhs_value),
            Star => lhs_value.multiply(&rhs_value),
            Slash => lhs_value.divide(&rhs_value),
            Power => lhs_value.power(&rhs_value),

            Equals => lhs_value.equals(&rhs_value),
            NotEquals => lhs_value.equals(&rhs_value).map(|value| Value {
                kind: ValueKind::Boolean(!value.is_truthy()),
                span: value.span,
            }),

            LessThan => lhs_value.less_than(&rhs_value),
            LessThanOrEqual => lhs_value.less_than_or_equal(&rhs_value),

            GreaterThan => lhs_value.greater_than(&rhs_value),
            GreaterThanOrEqual => lhs_value.greater_than_or_equal(&rhs_value),

            And => Ok(Value {
                kind: ValueKind::Boolean(lhs_value.is_truthy() && rhs_value.is_truthy()),
                span: (lhs_value.span.start..rhs_value.span.end).into(),
            }),

            Or => Ok(Value {
                kind: ValueKind::Boolean(lhs_value.is_truthy() || rhs_value.is_truthy()),
                span: (lhs_value.span.start..rhs_value.span.end).into(),
            }),

            _ => todo!(),
        }
    }
}
