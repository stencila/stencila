use neon::{
    context::{Context, FunctionContext},
    handle::Managed,
    result::JsResult,
    types::JsString,
};
use stencila::{
    errors::{self, Error},
    eyre, serde_json,
};

use crate::{prelude::to_json_or_throw, pubsub::bridging_subscriber};

/// Get the module's schemas
pub fn schema(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = errors::schema();
    to_json_or_throw(cx, schemas)
}

/// Publish a detailed error (including string `message` and other fields)
/// under the `errors` pubsub topic and throw a simple JavaScript error
/// with a message string
pub fn throw_error<T>(mut cx: FunctionContext, error: eyre::Report) -> JsResult<T>
where
    T: Managed,
{
    let message = error.to_string();

    let error = match error.downcast_ref::<Error>() {
        Some(error) => error,
        None => &Error::Unknown,
    };
    let error = match serde_json::to_value(error) {
        Ok(serde_json::Value::Object(mut value)) => {
            value.insert("message".into(), serde_json::Value::String(message.clone()));
            serde_json::Value::Object(value)
        }
        _ => serde_json::json!({ "message": message }),
    };
    // Send the error to the "errors" topic
    bridging_subscriber("errors".into(), error);

    // Throw it as a Javascript error
    cx.throw_error(message)
}
