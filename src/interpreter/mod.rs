mod data;
pub mod error;

use crate::{
    lexer::token::OperatorKind,
    parser::ast::{AstNode, AstNodeKind},
};

use self::{data::Value, error::InterpreterError};

/// A struct that represents an interpreter.
pub struct Interpreter {
    /// The AST that the interpreter will interpret.
    pub ast: AstNode,
}

// A type alias for the result of the interpreter.
type InterpreterResult<T> = Result<T, InterpreterError>;

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new(ast: AstNode) -> Self {
        Self { ast }
    }

    pub fn start(self) -> InterpreterResult<Value> {
        Interpreter::interpret(self.ast)
    }

    /// Interprets the AST.
    fn interpret(ast: AstNode) -> InterpreterResult<Value> {
        return match ast.kind {
            AstNodeKind::NumberLiteral(number) => Ok(Value::Number(number)),

            AstNodeKind::BinaryExpression { lhs, op, rhs } => {
                Ok(Interpreter::interpret_binary_expr(lhs, op, rhs)?)
            }

            _ => Ok(Value::Null),
        };
    }

    /// Interprets a binary expression.
    fn interpret_binary_expr(
        lhs: Box<AstNode>,
        op: OperatorKind,
        rhs: Box<AstNode>,
    ) -> InterpreterResult<Value> {
        let lhs_value = Interpreter::interpret(*lhs)?;
        let rhs_value = Interpreter::interpret(*rhs)?;

        use OperatorKind::*;
        use Value::*;

        match op {
            Plus => lhs_value.add(rhs_value),
            Minus => lhs_value.subtract(rhs_value),
            Star => lhs_value.multiply(rhs_value),
            Slash => lhs_value.divide(rhs_value),
            Power => lhs_value.power(rhs_value),

            LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual | And | Or => {
                let reducer = |a: f64, b: f64| match op {
                    LessThan => Boolean(a < b),
                    LessThanOrEqual => Boolean(a <= b),
                    GreaterThan => Boolean(a > b),
                    GreaterThanOrEqual => Boolean(a >= b),

                    _ => unreachable!(),
                };

                match op {
                    LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual => {
                        match (lhs_value, rhs_value) {
                            (Value::Number(lhs), Value::Number(rhs)) => {
                                return Ok(reducer(lhs, rhs))
                            }

                            _ => todo!(),
                        }
                    }

                    And => Ok(Boolean(lhs_value.is_truthy() && rhs_value.is_truthy())),
                    Or => Ok(Boolean(lhs_value.is_truthy() || rhs_value.is_truthy())),

                    _ => unreachable!(),
                }
            }

            _ => todo!(),
        }
    }
}
