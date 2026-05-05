pub mod error;
pub mod value;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{
        error::{Interrupt, RuntimeError, Signal},
        value::Value,
    },
    parser::ast::{Expression, Statement},
    source::{Span, Spanned},
};

type Result<T, E = Spanned<Interrupt>> = std::result::Result<T, E>;

/// A lexical environment in which bindings exist.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Environment {
    /// The enclosing parent [Environment].
    parent: Option<Rc<RefCell<Environment>>>,
    /// The variables bound in this environment.
    bindings: HashMap<&'static str, Value>,
}

/// A basic tree walking interpreter, responsible for evaluating source ASTs.
#[derive(Debug, Clone)]
pub struct Interpreter {
    /// The current environment.
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::default())),
        }
    }

    /// Executes a source file, running it until completion.
    pub fn execute(
        &mut self,
        tree: &Spanned<Statement>,
    ) -> Result<Option<Value>, Spanned<RuntimeError>> {
        self.statement(&tree.value, tree.span).map_err(|interrupt| {
            interrupt.map(|i| match i {
                Interrupt::Error(e) => e,
                Interrupt::Signal(s) => RuntimeError::from(s),
            })
        })
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

            // same as executing a block, but we don't pop the parent environment
            Statement::Repl { stmts, tail } => {
                for Spanned {
                    value: statement,
                    span,
                } in stmts
                {
                    self.statement(statement, *span)?;
                }

                if let Some(tail) = tail {
                    return Ok(Some(self.expression(&tail.value, tail.span)?.value));
                } else {
                    return Ok(Some(Value::Unit));
                }
            }

            Statement::Expression { expr, .. } => {
                return Ok(Some(self.expression(expr, span)?.value));
            }

            Statement::Print(expression) => {
                println!(
                    "{}",
                    self.expression(&expression.value, expression.span)?.value
                );
            }

            Statement::Assert(expression) => {
                let value = self.expression(&expression.value, expression.span)?.value;

                if !value.is_truthy() {
                    return Err(Spanned::wrap(
                        RuntimeError::AssertionFailed(value).into(),
                        expression.span,
                    ));
                }
            }

            Statement::Declaration {
                symbol,
                value: expr,
            } => {
                let environment = Environment::enclose(&self.environment);

                environment
                    .borrow_mut()
                    .bindings
                    .insert(*symbol, self.expression(&expr.value, expr.span)?.value);

                self.environment = environment;
            }

            Statement::While { predicate, body } => {
                while self
                    .expression(&predicate.value, predicate.span)?
                    .value
                    .is_truthy()
                {
                    if let Err(interrupt) = self.expression(&body.value, body.span) {
                        if interrupt.value == Interrupt::Signal(Signal::Break) {
                            break;
                        }

                        return Err(interrupt);
                    }
                }
            }

            Statement::Break => {
                return Err(Spanned::wrap(Signal::Break.into(), span));
            }

            Statement::FunctionDeclaration {
                symbol,
                parameters,
                body,
            } => {
                let environment = Environment::enclose(&self.environment);

                environment.borrow_mut().bindings.insert(
                    *symbol,
                    Value::Function {
                        parameters: parameters.clone(),
                        body: body.clone(),
                        enclosing: Rc::clone(&environment),
                        name: Some(*symbol),
                    },
                );

                self.environment = environment;
            }

            Statement::Return { result } => {
                let result = match result {
                    Some(expression) => self.expression(&expression.value, expression.span)?.value,
                    None => Value::Unit,
                };

                return Err(Spanned::wrap(Signal::Return(result).into(), span));
            }
        };

        Ok(None)
    }

    fn expression(&mut self, expression: &Expression, span: Span) -> Result<Spanned<Value>> {
        match expression {
            Expression::Integer(n) => Ok(Spanned::wrap(Value::Integer(*n), span)),

            Expression::Boolean(b) => Ok(Spanned::wrap(Value::Boolean(*b), span)),

            Expression::String(s) => Ok(Spanned::wrap(Value::String(s.to_owned()), span)),

            Expression::Variable { symbol } => {
                if let Some(value) = self.environment.borrow().search(symbol) {
                    Ok(Spanned::wrap(value, span))
                } else {
                    Err(Spanned::wrap(
                        RuntimeError::UnboundBinding { symbol }.into(),
                        span,
                    ))
                }
            }

            Expression::Assignment { symbol, expr } => {
                let value = self.expression(&expr.value, expr.span)?;

                self.environment
                    .borrow_mut()
                    .assign(symbol.value, value.value.clone())
                    .map(|_| value)
                    .map_err(|e| Spanned::wrap(e.into(), symbol.span))
            }

            Expression::BinaryOperation { lhs, operator, rhs } => {
                let lhs_result = self.expression(&lhs.value, lhs.span)?;
                let rhs_result = self.expression(&rhs.value, rhs.span)?;

                Value::binary_operation(lhs_result.value, *operator, rhs_result.value)
                    .map(|value| Spanned::wrap(value, span))
                    .map_err(|error| Spanned::wrap(error.into(), span))
            }

            Expression::UnaryOperation { operator, operand } => {
                let operand = self.expression(&operand.value, operand.span)?.value;

                Value::unary_operation(*operator, operand)
                    .map(|value| Spanned::wrap(value, span))
                    .map_err(|error| Spanned::wrap(error.into(), span))
            }

            Expression::Block { stmts, tail } => {
                let parent = Rc::clone(&self.environment);
                self.environment = Environment::enclose(&self.environment);

                for Spanned {
                    value: statement,
                    span,
                } in stmts
                {
                    self.statement(statement, *span)?;
                }

                let result = if let Some(tail) = tail {
                    self.expression(&tail.value, tail.span)
                } else {
                    Ok(Spanned::wrap(Value::Unit, span))
                };

                self.environment = parent;

                result
            }

            Expression::If {
                predicate,
                body,
                else_clause,
            } => {
                let predicate = self.expression(&predicate.value, predicate.span)?.value;

                if predicate.is_truthy() {
                    self.expression(&body.value, body.span)
                } else if let Some(else_clause) = else_clause {
                    self.expression(&else_clause.value, else_clause.span)
                } else {
                    Ok(Spanned::wrap(Value::Unit, span))
                }
            }

            Expression::Lambda { parameters, body } => {
                let environment = Environment::enclose(&self.environment);

                Ok(Spanned::wrap(
                    Value::Function {
                        name: None,
                        parameters: parameters.clone(),
                        body: (**body).clone(),
                        enclosing: environment,
                    },
                    span,
                ))
            }

            Expression::Call {
                callee: operand,
                arguments,
            } => {
                let parent = Rc::clone(&self.environment);

                let callee = self.expression(&operand.value, operand.span)?;

                let Value::Function {
                    parameters,
                    body,
                    enclosing,
                    name,
                } = callee.value
                else {
                    return Err(Spanned::wrap(
                        RuntimeError::NotCallable {
                            callee: callee.value,
                        }
                        .into(),
                        callee.span,
                    ));
                };

                // define all parameters
                if parameters.len() != arguments.len() {
                    return Err(Spanned::wrap(
                        RuntimeError::MismatchedArity {
                            name: name.unwrap_or("(anonymous)"),
                            expected: parameters.len(),
                            actual: arguments.len(),
                        }
                        .into(),
                        callee.span,
                    ));
                }

                // evaluate arguments in the caller's environment
                let mut evaluated = Vec::with_capacity(arguments.len());

                for argument in arguments {
                    evaluated.push(self.expression(&argument.value, argument.span)?.value);
                }

                // switch to the callee's environment
                self.environment = Environment::enclose(&enclosing);

                for (parameter, argument) in parameters.iter().zip(evaluated) {
                    self.environment
                        .borrow_mut()
                        .bindings
                        .insert(parameter.value, argument);
                }

                let result = self.expression(&body.value, body.span);
                self.environment = parent;

                if let Err(interrupt) = result {
                    if let Interrupt::Signal(Signal::Return(value)) = interrupt.value {
                        return Ok(Spanned::wrap(value, span));
                    }

                    // convert other signals into errors to prevent stack escaping
                    return Err(Spanned::wrap(
                        match interrupt.value {
                            Interrupt::Error(error) => error,
                            Interrupt::Signal(signal) => RuntimeError::from(signal),
                        }
                        .into(),
                        span,
                    ));
                }

                result
            }
        }
    }
}

impl Environment {
    /// Captures a parent [Environment], creating a new environment with no bindings set.
    pub fn enclose(parent: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            parent: Some(Rc::clone(parent)),
            bindings: HashMap::new(),
        }))
    }

    /// Recursively searches this [Environment] and its parent
    /// to find the value of the given `symbol`.
    pub fn search(&self, symbol: &'static str) -> Option<Value> {
        self.bindings.get(symbol).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|env| env.borrow().search(symbol))
        })
    }

    /// Attempts to assign an existing symbol to the closest enclosing scope.
    pub fn assign(&mut self, symbol: &'static str, value: Value) -> Result<(), RuntimeError> {
        if let Some(binding) = self.bindings.get_mut(symbol) {
            *binding = value;
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(symbol, value)
        } else {
            Err(RuntimeError::UnboundBinding { symbol })
        }
    }
}
