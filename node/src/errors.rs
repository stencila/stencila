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

/// Stop collecting errors
pub fn stop(cx: FunctionContext) -> JsResult<JsString> {
    let errors: Vec<serde_json::Value> = errors::stop()
        .iter()
        .map(|error| {
            let message = error.to_string();
            match serde_json::to_value(error) {
                Ok(serde_json::Value::Object(mut value)) => {
                    value.insert("message".into(), serde_json::Value::String(message.clone()));
                    serde_json::Value::Object(value)
                }
                _ => serde_json::json!({ "message": message }),
            }
        })
        .collect();
    to_json(cx, errors)
}

/// Throw an error as JSON so that it can be reconstituted in JavaScript
pub fn throw_error<T>(mut cx: FunctionContext, error: eyre::Report) -> JsResult<T>
where
    T: Managed,
{
    let error = match error.downcast_ref::<Error>() {
        Some(error) => error,
        None => &Error::Unknown,
    };
    let message = error.to_string();
    let error = match serde_json::to_value(error) {
        Ok(serde_json::Value::Object(mut value)) => {
            value.insert("message".into(), serde_json::Value::String(message.clone()));
            serde_json::Value::Object(value)
        }
        _ => serde_json::json!({ "message": message }),
    };
    let payload = match serde_json::to_string_pretty(&error) {
        Ok(json) => json,
        Err(_) => message,
    };
    cx.throw_error(payload)
}
