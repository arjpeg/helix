use crate::{
    compiler::{
        chunk::{Chunk, Constant, Instruction},
        error::{CompilerError, Result},
    },
    parser::ast::{BinaryOp, Expression, Statement, UnaryOp},
    source::{SourceMap, Span, Spanned},
};

pub mod chunk;
pub mod error;

/// The context of a compilation session of a [`Chunk`].
struct CompileCtx {
    /// All the local variables allocated thus far. May contain duplicates to
    /// support variable shadowing.
    locals: Vec<Local>,
    /// The current depth of scopes (how many blocks deep we are).
    scope_depth: usize,

    /// All errors accumulated so far.
    errors: Vec<Spanned<CompilerError>>,
}

/// A reference to a local variable.
#[derive(Debug, Clone)]
struct Local {
    /// The name of the variable.
    name: &'static str,
    /// The scope depth this local was first referenced at.
    scope_depth: usize,
}

/// Compiles a [`Statement::Program`] or [`Statement::Repl`] into an optimized [`Chunk`] ready for
/// execution by the virtual machine.
pub fn compile_program(program: Spanned<Statement>) -> Result<Chunk, Vec<Spanned<CompilerError>>> {
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

    Ok(chunk)
}

fn emit_statement(chunk: &mut Chunk, context: &mut CompileCtx, statement: Statement, span: Span) {
    match statement {
        Statement::Program { .. } | Statement::Repl { .. } => unreachable!(),

        Statement::Expression { expr, .. } => {
            emit_expression(chunk, context, expr, span);

            // clean stack after expression statements
            chunk.emit_instruction(Instruction::Pop, span);
        }

        Statement::Declaration {
            symbol,
            value: Spanned { value, span },
        } => {
            emit_expression(chunk, context, value, span);

            context.locals.push(Local {
                name: symbol,
                scope_depth: context.scope_depth,
            });
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
        Expression::String(_) => todo!(),

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
            let Some((index, _)) = context
                .locals
                .iter()
                .enumerate()
                .rev()
                .find(|(_, local)| local.name == symbol)
            else {
                context.errors.push(Spanned::wrap(
                    CompilerError::UnboundBinding { symbol },
                    span,
                ));

                return;
            };

            // due to the way stack is cleaned up after statements,
            // the indices from `locals` always match up to the location of elements on the stack,
            // so can just copy the same index
            chunk.emit_instruction(Instruction::GetLocal { index: index as u8 }, span);
        }

        Expression::Assignment { .. } => todo!(),
        Expression::List { .. } => todo!(),
        Expression::If { .. } => todo!(),
        Expression::Lambda { .. } => todo!(),
        Expression::Call { .. } => todo!(),
        Expression::Index { .. } => todo!(),
    };
}
