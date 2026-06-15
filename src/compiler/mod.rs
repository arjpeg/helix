use crate::{
    compiler::{
        chunk::{Chunk, Constant},
        error::{CompilerError, Result},
        instruction::Instruction,
    },
    interner::Symbol,
    parser::ast::{BinaryOp, Expression, LValue, Statement, UnaryOp},
    source::{SourceMap, Span, Spanned},
    vm::globals::Globals,
};

pub mod chunk;
pub mod error;
pub mod instruction;

/// The context of a compilation session of a [`Chunk`].
struct CompileCtx {
    /// All the local variables allocated thus far. May contain duplicates to
    /// support variable shadowing.
    locals: Vec<Local>,
    /// The current depth of scopes (how many blocks deep we are).
    scope_depth: usize,

    /// A set of all the known global variables declared.
    globals: Globals,

    /// All errors accumulated so far.
    errors: Vec<Spanned<CompilerError>>,
}

/// A reference to a local variable.
#[derive(Debug, Clone)]
struct Local {
    /// The name of the variable.
    name: Symbol,
    /// The scope depth this local was first referenced at.
    scope_depth: usize,
}

/// Compiles a [`Statement::Program`] or [`Statement::Repl`] into an optimized [`Chunk`] ready for
/// execution by the virtual machine.
pub fn compile_program(
    program: Spanned<Statement>,
    globals: &Globals,
) -> Result<(Chunk, Globals), Vec<Spanned<CompilerError>>> {
    let (stmts, tail) = match program.value {
        Statement::Program { stmts } => (stmts, None),
        Statement::Repl { stmts, tail } => (stmts, tail),
        _ => panic!("cannot compile non complete program statement"),
    };

    let name = SourceMap::get(program.span.source).path.to_str().unwrap();

    let mut chunk = Chunk::new(Some(name));
    let mut context = CompileCtx {
        locals: Vec::new(),
        scope_depth: 0,
        globals: globals.snapshot(),
        errors: Vec::new(),
    };

    for statement in stmts {
        emit_statement(&mut chunk, &mut context, statement.value, statement.span);
    }

    // emit one additional expression to the stack in REPL mode
    if let Some(expression) = tail {
        emit_expression(&mut chunk, &mut context, expression.value, expression.span);
    }

    chunk.emit_instruction(Instruction::Return, program.span);

    if !context.errors.is_empty() {
        return Err(context.errors);
    }

    Ok((chunk, context.globals))
}

fn emit_statement(chunk: &mut Chunk, context: &mut CompileCtx, statement: Statement, span: Span) {
    match statement {
        Statement::Program { .. } | Statement::Repl { .. } => unreachable!(),

        Statement::Expression { expr, .. } => {
            emit_expression(chunk, context, expr, span);

            // clean stack after expression statements
            chunk.emit_instruction(Instruction::Pop, span);
        }

        Statement::Declaration { symbol, value } => {
            emit_expression(chunk, context, value.value, value.span);

            // if we are scope_depth==0 (the global scope) declare the variable there instead
            if context.scope_depth == 0 {
                let symbol_index = chunk.emit_constant(Constant::Symbol(symbol));

                context.globals.known.insert(symbol);

                chunk.emit_instruction(
                    Instruction::DefineGlobal {
                        index: symbol_index,
                    },
                    span,
                );
            } else {
                // declare the variable as a normal local
                context.locals.push(Local {
                    name: symbol,
                    scope_depth: context.scope_depth,
                });
            }
        }

        Statement::Print(..) => todo!(),
        Statement::While { .. } => todo!(),
        Statement::Break => todo!(),
        Statement::Continue => todo!(),
        Statement::FunctionDeclaration { .. } => todo!(),
        Statement::Return { .. } => todo!(),
        Statement::Assert(..) => todo!(),
    };
}

fn emit_expression(
    chunk: &mut Chunk,
    context: &mut CompileCtx,
    expression: Expression,
    span: Span,
) {
    match expression {
        // constants
        Expression::Integer(i) => {
            let constant = chunk.emit_constant(Constant::from(i));
            chunk.emit_instruction(Instruction::LoadConstant { index: constant }, span);
        }
        Expression::Float(f) => {
            let constant = chunk.emit_constant(Constant::from(f));
            chunk.emit_instruction(Instruction::LoadConstant { index: constant }, span);
        }
        Expression::Boolean(b) => {
            let constant = chunk.emit_constant(Constant::from(b));
            chunk.emit_instruction(Instruction::LoadConstant { index: constant }, span);
        }
        Expression::String(s) => {
            let constant = chunk.emit_constant(Constant::from(&*s.leak()));
            chunk.emit_instruction(Instruction::LoadConstant { index: constant }, span);
        }

        Expression::BinaryOperation { lhs, operator, rhs } => {
            emit_expression(chunk, context, lhs.value, lhs.span);
            emit_expression(chunk, context, rhs.value, rhs.span);

            chunk.emit_instruction(
                match operator {
                    BinaryOp::Plus => Instruction::Add,
                    BinaryOp::Minus => Instruction::Subtract,
                    BinaryOp::Star => Instruction::Multiply,
                    BinaryOp::Slash => Instruction::Divide,
                    BinaryOp::NotEquals => todo!(),
                    BinaryOp::Equals => todo!(),
                    BinaryOp::GreaterThan => todo!(),
                    BinaryOp::GreaterThanEquals => todo!(),
                    BinaryOp::LessThan => todo!(),
                    BinaryOp::LessThanEquals => todo!(),
                    BinaryOp::And => todo!(),
                    BinaryOp::Or => todo!(),
                },
                span,
            );
        }

        Expression::UnaryOperation { operator, operand } => {
            emit_expression(chunk, context, operand.value, operand.span);

            // '+' is always a no op
            if operator == UnaryOp::Plus {
                return;
            }

            chunk.emit_instruction(
                match operator {
                    UnaryOp::Minus => Instruction::Negate,
                    UnaryOp::Bang => Instruction::Not,
                    UnaryOp::Plus => unreachable!(),
                },
                span,
            );
        }

        Expression::Block { stmts, tail } => {
            context.scope_depth += 1;

            for statement in stmts {
                emit_statement(chunk, context, statement.value, statement.span);
            }

            if let Some(expression) = tail {
                emit_expression(chunk, context, expression.value, expression.span);
            } else {
                // push a `()` onto the stack to properly indicate this block didn't have a tail
                let index = chunk.emit_constant(Constant::Unit);
                chunk.emit_instruction(Instruction::LoadConstant { index }, span);
            }

            // clean up all variables owned by this block
            while let Some(Local {
                scope_depth: local_depth,
                ..
            }) = context.locals.last()
                && *local_depth == context.scope_depth
            {
                let _ = context.locals.pop();
                chunk.emit_instruction(Instruction::Pop, span);
            }

            context.scope_depth -= 1;
        }

        Expression::Variable { symbol } => {
            if let Some(index) = find_local(context, symbol) {
                // due to the way stack is cleaned up after statements,
                // the indices from `locals` always match up to the location of elements on the stack,
                // so can just copy the same index
                chunk.emit_instruction(
                    Instruction::GetLocal {
                        stack_index: index as u8,
                    },
                    span,
                );
            } else if context.globals.known.contains(&symbol) {
                let symbol_index = chunk.emit_constant(Constant::Symbol(symbol));

                chunk.emit_instruction(
                    Instruction::GetGlobal {
                        name_index: symbol_index,
                    },
                    span,
                );
            } else {
                context.errors.push(Spanned::wrap(
                    CompilerError::UnboundBinding { symbol },
                    span,
                ));

                return;
            };
        }

        Expression::Assignment { target, value } => {
            emit_expression(chunk, context, value.value, value.span);

            match target.value {
                LValue::Variable(symbol) => {
                    if let Some(index) = find_local(context, symbol) {
                        chunk.emit_instruction(
                            Instruction::SetLocal {
                                stack_index: index as u8,
                            },
                            target.span,
                        );
                    } else if context.globals.known.contains(&symbol) {
                        let symbol_index = chunk.emit_constant(Constant::Symbol(symbol));

                        chunk.emit_instruction(
                            Instruction::GetGlobal {
                                name_index: symbol_index,
                            },
                            target.span,
                        );
                    } else {
                        context.errors.push(Spanned::wrap(
                            CompilerError::UnboundBinding { symbol },
                            target.span,
                        ));

                        return;
                    };
                }

                LValue::Index { .. } => todo!(),
            }
        }

        Expression::List { .. } => todo!(),
        Expression::If { .. } => todo!(),
        Expression::Lambda { .. } => todo!(),
        Expression::Call { .. } => todo!(),
        Expression::Index { .. } => todo!(),
    };
}

/// Returns the lastmost index of the given variable name in the [`CompileCtx::locals`].
fn find_local(context: &CompileCtx, symbol: Symbol) -> Option<usize> {
    context
        .locals
        .iter()
        .enumerate()
        .rev()
        .find(|(_, local)| local.name == symbol)
        .map(|(index, _)| index)
}
