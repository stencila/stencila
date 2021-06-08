use crate::errors::throw_error;
use neon::{prelude::*, result::Throw};
use stencila::{
    eyre,
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    serde_json,
    tokio::runtime::Runtime,
};

/// Convert a result to an `undefined` if it is OK, otherwise throw an error.
pub fn to_undefined_or_throw(
    mut cx: FunctionContext,
    result: eyre::Result<()>,
) -> JsResult<JsUndefined> {
    match result {
        Ok(_) => Ok(cx.undefined()),
        Err(error) => throw_error(cx, error),
    }
}

/// Convert a result to a string if it is OK, otherwise throw an error.
pub fn to_string_or_throw(
    mut cx: FunctionContext,
    result: eyre::Result<String>,
) -> JsResult<JsString> {
    match result {
        Ok(value) => Ok(cx.string(value)),
        Err(error) => throw_error(cx, error),
    }
}

// We currently JSON serialize / deserialize objects when passing them to / from Rust
// and Node.js. That's because, at the time of writing, the `neon-serde` crate
// (which provides a more convenient mechanism) was not compatible with the most
// recent `neon` version. These function just reduce some boilerplate associated with that.
// See https://github.com/neon-bindings/neon/pull/701 for progress on "native" neon serde
// compatibility.

/// Convert a value to JSON, throwing an error if that fails
pub fn to_json<Type>(mut cx: FunctionContext, value: Type) -> JsResult<JsString>
where
    Type: Serialize,
{
    match serde_json::to_string(&value) {
        Ok(json) => Ok(cx.string(json)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Convert a result to JSON if it is OK, otherwise throw an error
pub fn to_json_or_throw<Type>(cx: FunctionContext, result: eyre::Result<Type>) -> JsResult<JsString>
where
    Type: Serialize,
{
    match result {
        Ok(value) => to_json(cx, value),
        Err(error) => throw_error(cx, error),
    }
}

/// Convert JSON to a value
pub fn from_json<'a, Type>(cx: &mut FunctionContext, json: &'a str) -> Result<Type, Throw>
where
    Type: Deserialize<'a>,
{
    match serde_json::from_str::<Type>(&json) {
        Ok(value) => Ok(value),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// A global async runtime used to run any async functions
pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Unable to create runtime"));
