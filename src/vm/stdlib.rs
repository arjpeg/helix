use std::rc::Rc;

use crate::{
    interner::Interner,
    vm::{
        error::RuntimeError,
        globals::Globals,
        r#type::Type,
        value::{Native, Value},
    },
};

/// The default [`Globals`] environment to use when using the helix standard library.
pub fn default_environment() -> Globals {
    let fns = [
        Native {
            name: Interner::intern("push"),
            arity: Some(2),
            function: Rc::new(push),
        },
        Native {
            name: Interner::intern("length"),
            arity: Some(1),
            function: Rc::new(length),
        },
    ];

    let known = fns.iter().map(|f| f.name).collect();
    let runtime = fns
        .into_iter()
        .map(|f| (f.name, Value::Native(Rc::new(f))))
        .collect();

    Globals { known, runtime }
}

/// Pushes an item to the end of a list.
fn push(mut args: Vec<Value>) -> Result<Value, RuntimeError> {
    let value = args.pop().unwrap();

    let Value::List(list) = &args[0] else {
        return Err(RuntimeError::MismatchedType {
            name: Interner::intern("push"),
            n: Interner::intern("one"),
            received: Type::from(args[0].clone()),
        });
    };
    list.borrow_mut().push(value);

    Ok(Value::Unit)
}

/// Gets the length of a list or string.
fn length(mut args: Vec<Value>) -> Result<Value, RuntimeError> {
    let value = args.pop().unwrap();

    Ok(Value::Integer(match value {
        Value::String(s) => s.len(),
        Value::List(l) => l.borrow().len(),

        _ => {
            return Err(RuntimeError::MismatchedType {
                name: Interner::intern("length"),
                n: Interner::intern("one"),
                received: Type::from(value),
            });
        }
    } as i64))
}
