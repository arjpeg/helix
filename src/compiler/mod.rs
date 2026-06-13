use crate::{
    compiler::chunk::{Chunk, Constant, Instruction},
    parser::ast::{BinaryOp, Expression, Statement},
    source::{SourceMap, Span, Spanned},
};

pub mod chunk;

/// Compiles a [`Statement::Program`] into an optimized [`Chunk`] ready for virtual machine
/// execution.
pub fn compile_program(program: Spanned<Statement>) -> Chunk {
    let Statement::Program { stmts } = program.value else {
        panic!("cannot compile non complete program statement");
    };

    let name = SourceMap::get(program.span.source).path.to_str().unwrap();
    let mut chunk = Chunk::new(Some(name));

    for statement in stmts {
        emit_statement(&mut chunk, statement.value, statement.span);
    }

    chunk.emit_instruction(Instruction::Return, program.span);

    chunk
}

fn emit_statement(chunk: &mut Chunk, statement: Statement, span: Span) {
    match statement {
        Statement::Program { .. } | Statement::Repl { .. } => unreachable!(),

        Statement::Expression { expr, .. } => emit_expression(chunk, expr, span),

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
            chunk.emit_instruction(Instruction::Constant { index: constant }, span);
        }
        Expression::Float(f) => {
            let constant = chunk.emit_constant(Constant::from(f));
            chunk.emit_instruction(Instruction::Constant { index: constant }, span);
        }
        Expression::Boolean(b) => {
            let constant = chunk.emit_constant(Constant::from(b));
            chunk.emit_instruction(Instruction::Constant { index: constant }, span);
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

        Expression::Variable { .. } => todo!(),
        Expression::Assignment { .. } => todo!(),
        Expression::UnaryOperation { .. } => todo!(),
        Expression::List { .. } => todo!(),
        Expression::Block { .. } => todo!(),
        Expression::If { .. } => todo!(),
        Expression::Lambda { .. } => todo!(),
        Expression::Call { .. } => todo!(),
        Expression::Index { .. } => todo!(),
    };
}

