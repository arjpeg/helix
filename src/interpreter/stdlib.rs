use std::{
    collections::HashMap,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    interner::{Interner, Symbol},
    interpreter::{
        error::RuntimeError,
        value::{NativeFn, Value},
    },
    source::Spanned,
};

/// A type alias for the return signature of all [`NativeFn`]s.
type ValueResult = Result<Value, Spanned<RuntimeError>>;

/// Returns a [`HashMap`] with all the registered default items avaiable in the helix standard
/// library.
pub fn register() -> HashMap<Symbol, Value> {
    let functions = [
        NativeFn {
            name: Interner::intern("time"),
            arity: Some(0),
            function: Rc::new(time),
        },
        NativeFn {
            name: Interner::intern("len"),
            arity: Some(1),
            function: Rc::new(len),
        },
        NativeFn {
            name: Interner::intern("push"),
            arity: Some(2),
            function: Rc::new(push),
        },
    ];

    HashMap::from_iter(functions.map(|f| (f.name, Value::NativeFunction(f))))
}

fn time(_: Vec<Spanned<Value>>) -> ValueResult {
    Ok(Value::Integer(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as _,
    ))
}

fn len(params: Vec<Spanned<Value>>) -> ValueResult {
    match &params[0].value {
        Value::List(l) => Ok(Value::Integer(l.borrow().len() as _)),
        Value::String(s) => Ok(Value::Integer(s.chars().count() as _)),

        _ => Err(Spanned::wrap(
            RuntimeError::TypeError {
                name: "len",
                expected: "string or list",
                actual: params[0].value.type_name(),
            },
            params[0].span,
        )),
    }
}

fn push(mut params: Vec<Spanned<Value>>) -> ValueResult {
    let value = params.pop().unwrap().value;

    let Value::List(list) = &params[0].value else {
        return Err(Spanned::wrap(
            RuntimeError::TypeError {
                name: "push",
                expected: "list",
                actual: params[0].value.type_name(),
            },
            params[0].span,
        ));
    };

    list.borrow_mut().push(value);

    Ok(Value::Unit)
}
