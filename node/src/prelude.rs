use neon::{prelude::*, result::Throw};
use stencila::{
    eyre,
    serde::{Deserialize, Serialize},
    serde_json, tokio,
};

// We currently JSON serialize / deserialize objects when passing them to / from Rust
// and Node.js. That's because, at the time of writing, the `neon-serde` crate
// (which provides a more convenient mechanism) was not compatible with the most
// recent `neon` version. These function just reduce some boilerplate associated with that.
// See https://github.com/neon-bindings/neon/pull/701 for progress on "native" neon serde
// compatibility.

pub fn to_json<Type>(mut cx: FunctionContext, value: Type) -> JsResult<JsString>
where
    Type: Serialize,
{
    match serde_json::to_string(&value) {
        Ok(json) => Ok(cx.string(json)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn to_json_or_throw<Type>(
    mut cx: FunctionContext,
    result: eyre::Result<Type>,
) -> JsResult<JsString>
where
    Type: Serialize,
{
    match result {
        Ok(value) => to_json(cx, value),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn from_json<'a, Type>(cx: &mut FunctionContext, json: &'a str) -> Result<Type, Throw>
where
    Type: Deserialize<'a>,
{
    match serde_json::from_str::<Type>(&json) {
        Ok(value) => Ok(value),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Create a async runtime to await on async functions
pub fn runtime(cx: &mut FunctionContext) -> Result<tokio::runtime::Runtime, Throw> {
    match tokio::runtime::Runtime::new() {
        Ok(runtime) => Ok(runtime),
        Err(error) => cx.throw_error(error.to_string()),
    }
}
