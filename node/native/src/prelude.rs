use neon::{prelude::*, result::Throw};
use stencila::{
    serde::{Deserialize, Serialize},
    serde_json,
};

// We currently JSON serialize / deserialize objects when passing them to / from Rust
// and Node.js. That's because, at the time of writing, the `neon-serde` crate
// (which provides a more convenient mechanism) was not compatible with the most
// recent `neon` version. These function just reduce some boilerplate associated with that.

pub fn to_json<Type>(mut cx: FunctionContext, value: Type) -> JsResult<JsString>
where
    Type: Serialize,
{
    match serde_json::to_string(&value) {
        Ok(json) => Ok(cx.string(json)),
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
