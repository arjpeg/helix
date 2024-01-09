pub mod data;
pub mod error;
mod scope;

use std::{collections::hash_map::Entry, rc::Rc};

use crate::{
    lexer::{span::Span, token::OperatorKind},
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
                    Plus => Ok(expr),
                    Bang => Ok(Value {
                        kind: ValueKind::Boolean(!expr.is_truthy()),
                        span: ast.span,
                    }),
                    _ => unreachable!(),
                }
            }

            AstNodeKind::Assignment {
                name,
                value,
                declaration,
            } => {
                let value = self.interpret(*value)?;

                // Only allow expressions such as `x = 1` if `x` is already declared
                if !declaration {
                    for scope in self.scopes.iter_mut().rev() {
                        if let Entry::Occupied(mut e) = scope.variables.entry(name.clone()) {
                            e.insert(value);

                            return Ok(Value {
                                kind: ValueKind::Null,
                                span: ast.span,
                            });
                        }
                    }

                    return Err(InterpreterError::UndefinedVariable {
                        name,
                        span: ast.span,
                    });
                }

                self.current_scope().variables.insert(name, value);

                Ok(Value {
                    kind: ValueKind::Null,
                    span: ast.span,
                })
            }

            AstNodeKind::VariableReference(name) => {
                for scope in self.scopes.iter().rev() {
                    if let Some(value) = scope.variables.get(&name) {
                        return Ok(value.clone());
                    }
                }

                Err(InterpreterError::UndefinedVariable {
                    name,
                    span: ast.span,
                })
            }

            AstNodeKind::Block { expressions } => {
                self.scopes.push(Scope::new());

                let span = ast.span.clone();

                let mut return_value = Value {
                    kind: ValueKind::Null,
                    span,
                };

                for expression in expressions {
                    let ret = self.interpret(expression);

                    // Check for break and continue statements
                    match ret {
                        Ok(value) => {
                            return_value = value;
                        }
                        Err(InterpreterError::Break { .. } | InterpreterError::Continue { .. }) => {
                            return ret;
                        }
                        Err(err) => return Err(err),
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
                        kind: ValueKind::Null,
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
                    let ret = self.interpret(*body.clone());

                    // Check for break and continue errors
                    match ret {
                        Ok(value) => {
                            last_value = value;
                        }
                        Err(InterpreterError::Break { .. }) => {
                            break;
                        }
                        Err(InterpreterError::Continue { .. }) => {
                            continue;
                        }
                        Err(err) => return Err(err),
                    }
                }

                Ok(last_value)
            }

            AstNodeKind::Break => Err(InterpreterError::Break { span: ast.span }),
            AstNodeKind::Continue => Err(InterpreterError::Continue { span: ast.span }),

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
        let span: Span = (lhs.span.start..rhs.span.end, Rc::clone(&lhs.span.file)).into();

        let lhs_value = self.interpret(lhs)?;
        let rhs_value = self.interpret(rhs)?;

        use OperatorKind::*;

        match op {
            Bang | Assign => unreachable!("should be handled by unary op"),

            Plus => lhs_value.add(&rhs_value, span),
            Minus => lhs_value.subtract(&rhs_value, span),
            Star => lhs_value.multiply(&rhs_value, span),
            Slash => lhs_value.divide(&rhs_value, span),
            Power => lhs_value.power(&rhs_value, span),

            Equals => lhs_value.equals(&rhs_value, span),
            NotEquals => lhs_value.equals(&rhs_value, span).map(|value| Value {
                kind: ValueKind::Boolean(!value.is_truthy()),
                span: value.span,
            }),

            LessThan => lhs_value.less_than(&rhs_value, span),
            LessThanOrEqual => lhs_value.less_than_or_equal(&rhs_value, span),

            GreaterThan => lhs_value.greater_than(&rhs_value, span),
            GreaterThanOrEqual => lhs_value.greater_than_or_equal(&rhs_value, span),

            And => Ok(Value {
                kind: ValueKind::Boolean(lhs_value.is_truthy() && rhs_value.is_truthy()),
                span,
            }),

            Or => {
                if lhs_value.is_truthy() {
                    Ok(lhs_value)
                } else if rhs_value.is_truthy() {
                    Ok(rhs_value)
                } else {
                    Ok(Value {
                        kind: ValueKind::Boolean(false),
                        span,
                    })
                }
            }
        }
    }

    fn current_scope(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("no scope in stack ?!")
    }
}
