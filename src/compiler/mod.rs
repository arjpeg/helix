use crate::{
    compiler::chunk::{Chunk, Constant, Instruction},
    parser::ast::{BinaryOp, Expression, Statement, UnaryOp},
    source::{SourceMap, Span, Spanned},
};

pub mod chunk;

/// Compiles a [`Statement::Program`] or [`Statement::Repl`] into an optimized [`Chunk`] ready for
/// execution by the virtual machine.
pub fn compile_program(program: Spanned<Statement>) -> Chunk {
    let (stmts, tail) = match program.value {
        Statement::Program { stmts } => (stmts, None),
        Statement::Repl { stmts, tail } => (stmts, tail),
        _ => panic!("cannot compile non complete program statement"),
    };

    let name = SourceMap::get(program.span.source).path.to_str().unwrap();
    let mut chunk = Chunk::new(Some(name));

    for statement in stmts {
        emit_statement(&mut chunk, statement.value, statement.span);
    }

    // emit one additional expression to the stack in REPL mode
    if let Some(expression) = tail {
        emit_expression(&mut chunk, expression.value, expression.span);
    }

    chunk.emit_instruction(Instruction::Return, program.span);

    chunk
}

fn emit_statement(chunk: &mut Chunk, statement: Statement, span: Span) {
    match statement {
        Statement::Program { .. } | Statement::Repl { .. } => unreachable!(),

        Statement::Expression { expr, .. } => {
            emit_expression(chunk, expr, span);

            // clean stack after expression statements
            chunk.emit_instruction(Instruction::Pop, span);
        }

        Statement::Print(..) => todo!(),
        Statement::While { .. } => todo!(),
        Statement::Break => todo!(),
        Statement::Continue => todo!(),
        Statement::Declaration { .. } => todo!(),
        Statement::FunctionDeclaration { .. } => todo!(),
        Statement::Return { .. } => todo!(),
        Statement::Assert(..) => todo!(),
    }
}

fn emit_expression(chunk: &mut Chunk, expression: Expression, span: Span) {
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
            emit_expression(chunk, lhs.value, lhs.span);
            emit_expression(chunk, rhs.value, rhs.span);

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
            emit_expression(chunk, operand.value, operand.span);

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

        Expression::Variable { .. } => todo!(),
        Expression::Assignment { .. } => todo!(),
        Expression::List { .. } => todo!(),
        Expression::Block { .. } => todo!(),
        Expression::If { .. } => todo!(),
        Expression::Lambda { .. } => todo!(),
        Expression::Call { .. } => todo!(),
        Expression::Index { .. } => todo!(),
    };
}
