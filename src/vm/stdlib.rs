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
        Native {
            name: Interner::intern("string"),
            arity: Some(1),
            function: Rc::new(string),
        },
        Native {
            name: Interner::intern("integer"),
            arity: Some(1),
            function: Rc::new(integer),
        },
        Native {
            name: Interner::intern("float"),
            arity: Some(1),
            function: Rc::new(float),
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

/// Converts the provided argument into a [`Value::String`].
fn string(mut args: Vec<Value>) -> Result<Value, RuntimeError> {
    let value = args.pop().unwrap();

    if let Value::String(s) = value {
        Ok(Value::String(s))
    } else {
        Ok(Value::String(Rc::from(value.to_string())))
    }
}

/// Converts the provided argument into a [`Value::Integer`].
fn integer(mut args: Vec<Value>) -> Result<Value, RuntimeError> {
    let value = args.pop().unwrap();
    let ty = Type::from(&value);

    Ok(Value::Integer(match value {
        Value::Integer(n) => n,
        Value::Float(n) => n.round() as i64,
        Value::String(s) => s.parse().map_err(|_| RuntimeError::InvalidConversion {
            from: ty,
            to: Type::Integer,
        })?,

        _ => {
            return Err(RuntimeError::MismatchedType {
                name: Interner::intern("integer"),
                n: Interner::intern("one"),
                received: ty,
            });
        }
    }))
}

/// Converts the provided argument into a [`Value::Float`].
fn float(mut args: Vec<Value>) -> Result<Value, RuntimeError> {
    let value = args.pop().unwrap();
    let ty = Type::from(&value);

    Ok(Value::Float(match value {
        Value::Integer(n) => n as f64,
        Value::Float(n) => n,
        Value::String(s) => s.parse().map_err(|_| RuntimeError::InvalidConversion {
            from: ty,
            to: Type::Float,
        })?,

        _ => {
            return Err(RuntimeError::MismatchedType {
                name: Interner::intern("float"),
                n: Interner::intern("one"),
                received: ty,
            });
        }
    }))
}
