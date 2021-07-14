use neon::{
    context::{Context, FunctionContext},
    handle::Managed,
    result::JsResult,
    types::{JsString, JsUndefined},
};
use stencila::{
    errors::{self, Error},
    eyre, serde_json,
};

use crate::prelude::{to_json, to_json_or_throw};

/// Get the module's schemas
pub fn schema(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = errors::schema();
    to_json_or_throw(cx, schemas)
}

/// Start collecting errors
pub fn start(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    errors::start();
    Ok(cx.undefined())
}

/// Stop collecting errors and return them as a JSON array
pub fn stop(cx: FunctionContext) -> JsResult<JsString> {
    let errors: Vec<serde_json::Value> = errors::stop()
        .iter()
        .map(|error| error_to_json(error))
        .collect();
    to_json(cx, errors)
}

/// Throw an error as JSON
pub fn throw_error<T>(mut cx: FunctionContext, error: eyre::Report) -> JsResult<T>
where
    T: Managed,
{
    let message = error.to_string();
    let value = match error.downcast_ref::<Error>() {
        Some(error) => error_to_json(error),
        None => error_to_json(&Error::Unspecified { message }),
    };
    let json = serde_json::to_string_pretty(&value).expect("To always be able to stringify");
    cx.throw_error(json)
}

/// Convert an error to a JSON value so that it can be reconstituted in JavaScript
fn error_to_json(error: &Error) -> serde_json::Value {
    let message = error.to_string();
    match serde_json::to_value(error) {
        Ok(serde_json::Value::Object(mut value)) => {
            value.insert("message".into(), serde_json::Value::String(message));
            serde_json::Value::Object(value)
        }
        _ => serde_json::json!({ "message": message }),
    }
}
