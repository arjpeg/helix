pub mod data;
pub mod error;
mod scope;

use crate::{
    lexer::token::OperatorKind,
    parser::ast::{AstNode, AstNodeKind},
};

use self::{
    data::{Value, ValueKind},
    error::InterpreterError,
    scope::Scope,
};

/// A struct that represents an interpreter.
pub struct Interpreter {
    /// The stack of scopes.
    scopes: Vec<Scope>,
}

// A type alias for the result of the interpreter.
type InterpreterResult<T> = Result<T, InterpreterError>;

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    pub fn start(&mut self, ast: AstNode) -> InterpreterResult<Value> {
        self.interpret(ast)
    }

    /// Interprets the AST.
    pub fn interpret(&mut self, ast: AstNode) -> InterpreterResult<Value> {
        match ast.kind {
            AstNodeKind::NumberLiteral(number) => Ok(Value {
                kind: ValueKind::Number(number),
                span: ast.span,
            }),

            AstNodeKind::StringLiteral(string) => Ok(Value {
                kind: ValueKind::String(string),
                span: ast.span,
            }),

            AstNodeKind::BinaryExpression { lhs, op, rhs } => {
                Ok(self.interpret_binary_expr(*lhs, op, *rhs)?)
            }

            AstNodeKind::UnaryExpression { op, expr } => {
                let expr = self.interpret(*expr)?;

                use OperatorKind::*;
                match op {
                    Minus => expr.negate(),
                    Bang => Ok(Value {
                        kind: ValueKind::Boolean(!expr.is_truthy()),
                        span: ast.span,
                    }),
                    _ => unreachable!(),
                }
            }

            AstNodeKind::Assignment { name, value } => {
                let value = self.interpret(*value)?;
                self.current_scope().variables.insert(name, value.clone());

                Ok(value)
            }

            AstNodeKind::VariableReference(name) => {
                Ok(self.current_scope().variables.get(&name).cloned().ok_or({
                    InterpreterError::UndefinedVariable {
                        name,
                        span: ast.span,
                    }
                })?)
            }

            AstNodeKind::Block { expressions } => {
                self.scopes.push(Scope::new());

                let mut return_value = Value {
                    kind: ValueKind::Null,
                    span: ast.span,
                };

                for expression in expressions {
                    return_value = self.interpret(expression)?;

                    if self.current_scope().should_break {
                        break;
                    }

                    if self.current_scope().should_continue {
                        self.current_scope().should_continue = false;
                        continue;
                    }

                    if let Some(ret_value) = &self.current_scope().return_value {
                        return_value = ret_value.clone();
                        break;
                    }
                }

                self.scopes.pop();

                Ok(return_value)
            }

            AstNodeKind::If {
                condition,
                body,
                else_branch,
            } => {
                let condition = self.interpret(*condition)?;

                if condition.is_truthy() {
                    self.interpret(*body)
                } else if let Some(else_branch) = else_branch {
                    self.interpret(*else_branch)
                } else {
                    Ok(Value {
                        kind: ValueKind::Boolean(false),
                        span: ast.span,
                    })
                }
            }

            AstNodeKind::Else { body } => self.interpret(*body),

            AstNodeKind::Print { expression } => {
                let value = self.interpret(*expression)?;
                println!("{}", value.kind);

                Ok(Value {
                    kind: ValueKind::Null,
                    span: ast.span,
                })
            }

            AstNodeKind::While { condition, body } => {
                let mut last_value = Value {
                    kind: ValueKind::Null,
                    span: ast.span,
                };

                while self.interpret(*condition.clone())?.is_truthy() {
                    last_value = self.interpret(*body.clone())?;
                }

                Ok(last_value)
            }

            AstNodeKind::NoOp => Ok(Value {
                kind: ValueKind::Null,
                span: ast.span,
            }),

            AstNodeKind::FunctionDefinition { params, body, name } => {
                let value = Value {
                    kind: ValueKind::Function {
                        name: name.clone(),
                        parameters: params,
                        body: *body,
                    },
                    span: ast.span,
                };

                self.current_scope().variables.insert(name, value.clone());

                Ok(value)
            }
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
                span: (
                    lhs_value.span.start..rhs_value.span.end,
                    lhs_value.span.file,
                )
                    .into(),
            }),

            Or => Ok(Value {
                kind: ValueKind::Boolean(lhs_value.is_truthy() || rhs_value.is_truthy()),
                span: (
                    lhs_value.span.start..rhs_value.span.end,
                    lhs_value.span.file,
                )
                    .into(),
            }),

            _ => todo!(),
        }
    }

    fn current_scope(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("no scope in stack ?!")
    }
}
