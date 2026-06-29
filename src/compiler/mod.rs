use crate::{
    compiler::{
        chunk::{Chunk, Function},
        constants::Constant,
        error::{CompilerError, Result},
        index::{ConstantIndex, LocalIndex, StackIndex, UpvalueIndex},
        instruction::Instruction,
    },
    interner::{Interner, Symbol},
    parser::ast::{BinaryOp, Expression, LValue, Statement, UnaryOp},
    source::{SourceMap, Span, Spanned},
    vm::globals::Globals,
};

pub mod chunk;
pub mod constants;
pub mod error;
pub mod index;
pub mod instruction;

/// The context of compiling a complete program.
struct CompileCtx {
    /// A set of all the known global variables declared.
    globals: Globals,
    /// The stack of [`FunctionCtx`]s (last = innermost compiled function).
    functions: Vec<FunctionCtx>,

    /// All errors accumulated so far.
    errors: Vec<Spanned<CompilerError>>,
}

/// The state of compilation in a function, or the global script.
struct FunctionCtx {
    /// The current [`Chunk`] buffer associated with this function.
    chunk: Chunk,

    /// The name of this function.
    name: Option<Symbol>,
    /// The arity of this function (if applicable).
    arity: Option<u8>,

    /// All the local variables allocated thus far. May contain duplicates to
    /// support variable shadowing.
    locals: Vec<Local>,
    /// All the upvalues captured by this function. Will not contain any duplicates as upvalues all
    /// capture the same binding.
    upvalues: Vec<Upvalue>,
    /// The current depth of scopes (how many blocks deep we are).
    scope_depth: usize,

    /// The list of addresses of `break` instructions that need to be backpatched (last = innermost).
    break_addresses: Vec<Vec<usize>>,
    /// The list of addresses of `continue` instructions that need to be backpatched (last = innermost).
    continue_addresses: Vec<Vec<usize>>,
}

/// The different locations of a variable binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Binding {
    /// A reference to a local variable, offset from the base slot by `.0`.
    Local(LocalIndex),
    /// A reference to a captured local variable, holding the upvalue number.
    Upvalue(UpvalueIndex),
    /// A reference to a global variable, holding the index of the symbol in the constant pool.
    Global(ConstantIndex),
}

/// A reference to a local variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Local {
    /// The name of the variable.
    name: Symbol,
    /// The scope depth this local was first referenced at.
    scope_depth: usize,
}

/// A reference to a captured local variable from an enclosing [`FunctionCtx`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Upvalue {
    /// A binding that was directly captured as a local variable from the enclosing function.
    Local(LocalIndex),
    /// A binding that was captured as an upvalue of the enclosing function.
    Transitive(UpvalueIndex),
}

impl CompileCtx {
    /// Returns an immutable reference to the innermost [`FunctionCtx`].
    fn current(&self) -> &FunctionCtx {
        self.functions
            .last()
            .expect("should never stack underflow of function contexts")
    }

    /// Returns a mutable reference to the innermost [`FunctionCtx`].
    fn current_mut(&mut self) -> &mut FunctionCtx {
        self.functions
            .last_mut()
            .expect("should never stack underflow of function contexts")
    }

    /// Returns an immutable reference to the innermost [`Chunk`].
    fn chunk(&self) -> &Chunk {
        &self.current().chunk
    }

    /// Returns a mutable reference to the innermost [`Chunk`].
    fn chunk_mut(&mut self) -> &mut Chunk {
        &mut self.current_mut().chunk
    }
}

/// Compiles a [`Statement::Program`] or [`Statement::Repl`] into an optimized [`Function`] ready for
/// execution by the virtual machine.
pub fn compile_program(
    program: Spanned<Statement>,
    globals: &Globals,
) -> Result<(Function, Globals), Vec<Spanned<CompilerError>>> {
    let (stmts, tail) = match program.value {
        Statement::Program { stmts } => (stmts, None),
        Statement::Repl { stmts, tail } => (stmts, tail),
        _ => panic!("cannot compile non complete program statement"),
    };

    let name = Interner::intern(SourceMap::get(program.span.source).path.to_str().unwrap());

    let script = FunctionCtx {
        chunk: Chunk::new(Some(name)),
        name: Some(name),
        arity: None,
        locals: Vec::new(),
        upvalues: Vec::new(),
        scope_depth: 0,
        break_addresses: Vec::new(),
        continue_addresses: Vec::new(),
    };

    let mut context = CompileCtx {
        functions: vec![script],
        globals: globals.snapshot(),
        errors: Vec::new(),
    };

    for statement in stmts {
        emit_statement(&mut context, statement.value, statement.span);
    }

    // emit one additional expression to the stack in REPL mode
    if let Some(expression) = tail {
        emit_expression(&mut context, expression.value, expression.span);
    }

    context
        .chunk_mut()
        .emit_instruction(Instruction::Return, program.span);

    if !context.errors.is_empty() {
        return Err(context.errors);
    }

    // extract the innermost chunk
    let script = context.functions.pop().unwrap();

    assert!(
        context.functions.is_empty(),
        "compile context should be empty after compilation"
    );

    Ok((Function::from(script), context.globals))
}

fn emit_statement(context: &mut CompileCtx, statement: Statement, span: Span) -> usize {
    let start = context.chunk().code.len();

    match statement {
        Statement::Program { .. } | Statement::Repl { .. } => unreachable!(),

        Statement::Expression { expr, .. } => {
            emit_expression(context, expr, span);

            // clean stack after expression statements
            context.chunk_mut().emit_instruction(Instruction::Pop, span);
        }

        Statement::Declaration { symbol, value } => {
            emit_expression(context, value.value, value.span);
            declare_binding(context, symbol);
            define_binding(context, symbol, span);
        }

        Statement::While { predicate, body } => {
            // $START:
            // predicate
            // JUMP_IF_FALSE $END
            // body
            // JUMP $START
            // $END:

            let loop_start = emit_expression(context, predicate.value, predicate.span);
            let jump_to_end = context
                .chunk_mut()
                .emit_instruction(Instruction::JumpIfFalse { offset: 0 }, span);

            context.current_mut().break_addresses.push(Vec::new());
            context.current_mut().continue_addresses.push(Vec::new());

            emit_expression(context, body.value, body.span);
            // clean stack from the expression
            context
                .chunk_mut()
                .emit_instruction(Instruction::Pop, body.span);

            let jump_to_start = context
                .chunk_mut()
                .emit_instruction(Instruction::Jump { offset: 0 }, span);
            context
                .chunk_mut()
                .backpatch_jump(jump_to_start, Some(loop_start));
            context.chunk_mut().backpatch_jump(jump_to_end, None);

            let break_backpatches = context.current_mut().break_addresses.pop().unwrap();
            let continue_backpatches = context.current_mut().continue_addresses.pop().unwrap();

            for break_address in break_backpatches {
                context.chunk_mut().backpatch_jump(break_address, None);
            }

            for continue_address in continue_backpatches {
                context
                    .chunk_mut()
                    .backpatch_jump(continue_address, Some(loop_start));
            }
        }

        Statement::Print(expression) => {
            emit_expression(context, expression.value, expression.span);
            context
                .chunk_mut()
                .emit_instruction(Instruction::Print, span);
        }

        Statement::Break => {
            let jump = context
                .chunk_mut()
                .emit_instruction(Instruction::Jump { offset: 0 }, span);

            let Some(loop_ctx) = context.current_mut().break_addresses.last_mut() else {
                context
                    .errors
                    .push(Spanned::new(CompilerError::Break, span));

                return start;
            };

            loop_ctx.push(jump);
        }

        Statement::Continue => {
            let jump = context
                .chunk_mut()
                .emit_instruction(Instruction::Jump { offset: 0 }, span);

            let Some(loop_ctx) = context.current_mut().continue_addresses.last_mut() else {
                context
                    .errors
                    .push(Spanned::new(CompilerError::Continue, span));

                return start;
            };

            loop_ctx.push(jump);
        }

        Statement::FunctionDeclaration {
            symbol: name,
            parameters,
            body,
        } => {
            compile_function(context, Some(name), parameters, body, span);
        }

        Statement::Return { result } => {
            if let Some(return_value) = result {
                emit_expression(context, return_value.value, return_value.span);
            } else {
                let chunk = context.chunk_mut();
                let index = chunk.emit_constant(Constant::Unit);
                chunk.emit_instruction(Instruction::LoadConstant(index), span);
            }

            context
                .chunk_mut()
                .emit_instruction(Instruction::Return, span);
        }

        Statement::Assert(expression) => {
            emit_expression(context, expression.value, expression.span);

            context
                .chunk_mut()
                .emit_instruction(Instruction::Assert, span);
        }
    };

    start
}

fn emit_expression(context: &mut CompileCtx, expression: Expression, span: Span) -> usize {
    let start = context.chunk().code.len();

    match expression {
        // constants
        Expression::Integer(i) => {
            let constant = context.chunk_mut().emit_constant(Constant::from(i));
            context
                .chunk_mut()
                .emit_instruction(Instruction::LoadConstant(constant), span);
        }
        Expression::Float(f) => {
            let constant = context.chunk_mut().emit_constant(Constant::from(f));
            context
                .chunk_mut()
                .emit_instruction(Instruction::LoadConstant(constant), span);
        }
        Expression::Boolean(b) => {
            let constant = context.chunk_mut().emit_constant(Constant::from(b));
            context
                .chunk_mut()
                .emit_instruction(Instruction::LoadConstant(constant), span);
        }
        Expression::String(s) => {
            let constant = context
                .chunk_mut()
                .emit_constant(Constant::from(&*s.leak()));
            context
                .chunk_mut()
                .emit_instruction(Instruction::LoadConstant(constant), span);
        }

        Expression::BinaryOperation { lhs, operator, rhs } => {
            // to properly handle short circuiting for logical operations, we implement
            // 'and' and 'or' in a special way

            // A && B and A ||B compiles to
            // A
            // DUPLICATE // store A temporarily
            // JUMP_IF_XXXX L1 // leaves A whatever the value of A was
            // POP
            // B // leaves B (true or false) since it determines the value of the operation
            // L1
            //
            // where XXX = false if op = and, and XXX = true if op = or
            if operator == BinaryOp::And || operator == BinaryOp::Or {
                emit_expression(context, lhs.value, lhs.span);
                context
                    .chunk_mut()
                    .emit_instruction(Instruction::Duplicate, lhs.span);

                let jump_instruction = if operator == BinaryOp::And {
                    Instruction::JumpIfFalse { offset: 0 }
                } else {
                    Instruction::JumpIfTrue { offset: 0 }
                };

                let jump = context.chunk_mut().emit_instruction(jump_instruction, span);
                context
                    .chunk_mut()
                    .emit_instruction(Instruction::Pop, lhs.span);

                emit_expression(context, rhs.value, rhs.span);

                context.chunk_mut().backpatch_jump(jump, None);

                return start;
            }

            // swap order arguments are placed on the stack for these two operators,
            // as a > b is equivalent to b < a
            if operator == BinaryOp::GreaterThan || operator == BinaryOp::GreaterThanEquals {
                emit_expression(context, rhs.value, rhs.span);
                emit_expression(context, lhs.value, lhs.span);
            } else {
                emit_expression(context, lhs.value, lhs.span);
                emit_expression(context, rhs.value, rhs.span);
            }

            context.chunk_mut().emit_instruction(
                match operator {
                    BinaryOp::Plus => Instruction::Add,
                    BinaryOp::Minus => Instruction::Subtract,
                    BinaryOp::Star => Instruction::Multiply,
                    BinaryOp::Slash => Instruction::Divide,
                    BinaryOp::Equals => Instruction::Equals,
                    // we correct not equals below
                    BinaryOp::NotEquals => Instruction::Equals,
                    BinaryOp::GreaterThan => Instruction::LessThan,
                    BinaryOp::GreaterThanEquals => Instruction::LessThanEquals,
                    BinaryOp::LessThan => Instruction::LessThan,
                    BinaryOp::LessThanEquals => Instruction::LessThanEquals,

                    // handled above
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                },
                span,
            );

            // correct for not equals, as a != b is equivalent to !(a == b)
            if operator == BinaryOp::NotEquals {
                context.chunk_mut().emit_instruction(Instruction::Not, span);
            }
        }

        Expression::UnaryOperation { operator, operand } => {
            emit_expression(context, operand.value, operand.span);

            match operator {
                UnaryOp::Minus => context
                    .chunk_mut()
                    .emit_instruction(Instruction::Negate, span),
                UnaryOp::Bang => context.chunk_mut().emit_instruction(Instruction::Not, span),
                // '+' is always a no op
                UnaryOp::Plus => 0,
            };
        }

        Expression::Block { stmts, tail } => {
            context.current_mut().scope_depth += 1;

            for statement in stmts {
                emit_statement(context, statement.value, statement.span);
            }

            if let Some(expression) = tail {
                emit_expression(context, expression.value, expression.span);
            } else {
                // push a `()` onto the stack to keep the stack balanced
                let index = context.chunk_mut().emit_constant(Constant::Unit);
                context
                    .chunk_mut()
                    .emit_instruction(Instruction::LoadConstant(index), span);
            }

            let current_depth = context.current().scope_depth;

            // clean up all locals owned by this block
            let scope_local_count = context
                .current()
                .locals
                .iter()
                .rev()
                .take_while(|l| l.scope_depth == current_depth)
                .count();

            if scope_local_count > 0 {
                let new_size = context.current().locals.len() - scope_local_count;
                context.current_mut().locals.truncate(new_size);

                context.chunk_mut().emit_instruction(
                    Instruction::CloseAbove {
                        from: StackIndex(new_size.saturating_sub(1) as u16),
                    },
                    span,
                );

                context.chunk_mut().emit_instruction(
                    Instruction::PopUnder {
                        n: scope_local_count as u8,
                    },
                    span,
                );
            }

            context.current_mut().scope_depth -= 1;
        }

        Expression::Variable { symbol } => {
            let Some(binding) = resolve_binding(context, symbol, span) else {
                return start;
            };

            context.chunk_mut().emit_instruction(binding.get(), span);
        }

        Expression::Assignment { target, value } => {
            emit_expression(context, value.value, value.span);

            context
                .chunk_mut()
                .emit_instruction(Instruction::Duplicate, value.span);

            match target.value {
                LValue::Variable(symbol) => {
                    let Some(binding) = resolve_binding(context, symbol, span) else {
                        return start;
                    };

                    context.chunk_mut().emit_instruction(binding.set(), span);
                }

                LValue::Index { .. } => todo!(),
            }
        }

        Expression::If {
            predicate,
            body,
            else_clause,
        } => {
            emit_expression(context, predicate.value, predicate.span);

            let else_jump = context
                .chunk_mut()
                .emit_instruction(Instruction::JumpIfFalse { offset: 0 }, span);

            emit_expression(context, body.value, body.span);

            // the end of the entire if (else) chain
            let end_jump = context
                .chunk_mut()
                .emit_instruction(Instruction::Jump { offset: 0 }, span);

            context.chunk_mut().backpatch_jump(else_jump, None); // skip past the jump at the end of the if block

            if let Some(expression) = else_clause {
                // write the else block
                emit_expression(context, expression.value, expression.span);
            } else {
                // keep consistent stack
                let c = context.chunk_mut().emit_constant(Constant::Unit);

                context
                    .chunk_mut()
                    .emit_instruction(Instruction::LoadConstant(c), span);
            }

            context.chunk_mut().backpatch_jump(end_jump, None);
        }

        Expression::Lambda { parameters, body } => {
            compile_function(context, None, parameters, *body, span);
        }

        Expression::Call { callee, arguments } => {
            emit_expression(context, callee.value, callee.span);

            let n_arguments = u8::try_from(arguments.len()).unwrap();

            // the `n`th argument corresponds to the `n`th parameter
            for argument in arguments {
                emit_expression(context, argument.value, argument.span);
            }

            context.chunk_mut().emit_instruction(
                Instruction::Call {
                    arguments: n_arguments,
                },
                span,
            );
        }

        Expression::List { .. } => todo!(),
        Expression::Index { .. } => todo!(),
    };

    start
}

/// Declares a local or global variable based on the current scope depth.
///
/// If the `context.current().scope_depth` == 0, the value is defined as a global,
/// else as a local
fn declare_binding(context: &mut CompileCtx, symbol: Symbol) {
    // if we are scope_depth==0 (the global scope) declare the variable there instead
    if context.current().scope_depth == 0 {
        context.globals.known.insert(symbol);
    } else {
        // declare the variable as a normal local
        let current_depth = context.current().scope_depth;

        context.current_mut().locals.push(Local {
            name: symbol,
            scope_depth: current_depth,
        });
    }
}

/// Compiles a function definition.
fn compile_function(
    context: &mut CompileCtx,
    name: Option<Symbol>,
    parameters: Vec<Spanned<Symbol>>,
    body: Spanned<Expression>,
    span: Span,
) {
    let arity = u8::try_from(parameters.len()).unwrap();

    // "" is not a valid identifer, so it is safe to use
    let slot_zero = name.unwrap_or_else(|| Interner::intern(""));

    // declare binding for outer scopes
    declare_binding(context, slot_zero);

    context.functions.push(FunctionCtx {
        chunk: Chunk::new(name),
        name: name,
        arity: Some(arity),
        locals: Vec::new(),
        upvalues: Vec::new(),
        scope_depth: 1,
        break_addresses: Vec::new(),
        continue_addresses: Vec::new(),
    });

    // declare binding for inner scope to allow recursion
    declare_binding(context, slot_zero);

    // define all parameters
    for parameter in parameters {
        declare_binding(context, parameter.value);
    }

    emit_expression(context, body.value, body.span);

    context
        .chunk_mut()
        .emit_instruction(Instruction::Return, span);

    let compiled = context.functions.pop().unwrap();

    let chunk = context.chunk_mut();
    let function = chunk.emit_function(Function::from(compiled));
    chunk.emit_instruction(Instruction::MakeClosure(function), span);

    define_binding(context, slot_zero, span);
}

/// Defines a local or global variable based on the current scope depth.
///
/// For local variables, this function has no effect, but for global variables,
/// the top most value on the stack is popped and used as the value for the symbol.
fn define_binding(context: &mut CompileCtx, symbol: Symbol, span: Span) {
    if context.current().scope_depth != 0 {
        return;
    }

    let name = context.chunk_mut().emit_constant(Constant::Symbol(symbol));

    context
        .chunk_mut()
        .emit_instruction(Instruction::DefineGlobal(name), span);
}

/// Attempts to resolve the [`Binding`] location for the given symbol.
///
/// Emits an error if it wasn't found.
fn resolve_binding(context: &mut CompileCtx, symbol: Symbol, span: Span) -> Option<Binding> {
    // due to the way stack is cleaned up after statements,
    // the indices from `locals` always match up to the location of elements on the stack,
    // so we can just copy the same index
    if let Some(index) = resolve_local(context.current(), symbol) {
        Some(Binding::Local(index))
    } else if let Some(index) = resolve_upvalue(context, symbol, context.functions.len() - 1) {
        Some(Binding::Upvalue(index))
    } else if context.globals.known.contains(&symbol) {
        let symbol_index = context.chunk_mut().emit_constant(Constant::Symbol(symbol));
        Some(Binding::Global(symbol_index))
    } else {
        context
            .errors
            .push(Spanned::new(CompilerError::UnboundBinding { symbol }, span));

        None
    }
}

/// Returns the lastmost index of the given variable name in the given [`FunctionCtx::locals`].
fn resolve_local(function: &FunctionCtx, symbol: Symbol) -> Option<LocalIndex> {
    function
        .locals
        .iter()
        .enumerate()
        .rev()
        .find(|(_, local)| local.name == symbol)
        .map(|(index, _)| LocalIndex(u8::try_from(index).unwrap()))
}

/// Returns the upvalue index of the given variable name, recurursing up from the given
/// `frame_index`.
fn resolve_upvalue(
    context: &mut CompileCtx,
    symbol: Symbol,
    frame_index: usize,
) -> Option<UpvalueIndex> {
    // we hit the global scope, there can't be any upvalues to capture
    if frame_index == 0 {
        return None;
    }

    // attempt to search the enclosing locals
    if let Some(local) = resolve_local(&context.functions[frame_index - 1], symbol) {
        return Some(add_upvalue(context, frame_index, Upvalue::Local(local)));
    }

    // attempt to search the enclosing upvalues
    if let Some(upvalue) = resolve_upvalue(context, symbol, frame_index - 1) {
        return Some(add_upvalue(
            context,
            frame_index,
            Upvalue::Transitive(upvalue),
        ));
    }

    None
}

/// Adds an upvalue to the given function indexed by `frame_index`.
fn add_upvalue(context: &mut CompileCtx, frame_index: usize, upvalue: Upvalue) -> UpvalueIndex {
    let frame = &mut context.functions[frame_index];

    // search if this upvalue has already been captured within this frame
    let index = frame
        .upvalues
        .iter()
        .position(|other| upvalue == *other)
        .unwrap_or_else(|| {
            let index = frame.upvalues.len();
            frame.upvalues.push(upvalue);
            index
        });

    UpvalueIndex(u8::try_from(index).unwrap())
}

impl Binding {
    /// Returns the appropriate [`Instruction`] to read from this binding.
    fn get(self) -> Instruction {
        match self {
            Binding::Local(stack_index) => Instruction::GetLocal(stack_index),
            Binding::Upvalue(upvalue_index) => Instruction::GetUpvalue(upvalue_index),
            Binding::Global(name_index) => Instruction::GetGlobal(name_index),
        }
    }

    /// Returns the appropriate [`Instruction`] to set from this binding.
    fn set(self) -> Instruction {
        match self {
            Binding::Local(stack_index) => Instruction::SetLocal(stack_index),
            Binding::Upvalue(upvalue_index) => Instruction::SetUpvalue(upvalue_index),
            Binding::Global(name_index) => Instruction::SetGlobal(name_index),
        }
    }
}

impl From<FunctionCtx> for Function {
    fn from(value: FunctionCtx) -> Self {
        let FunctionCtx {
            chunk,
            name,
            arity,
            upvalues,
            ..
        } = value;

        Self {
            arity: arity.unwrap_or_default(),
            upvalues,
            chunk,
            name,
        }
    }
}
