use crate::prelude::*;
use neon::prelude::*;
use stencila::{
    config::{self, Config, CONFIG},
    serde_json,
    tokio::sync::MutexGuard,
    validator::Validate,
};

/// Lock the global config store
pub fn lock(cx: &mut FunctionContext) -> NeonResult<MutexGuard<'static, Config>> {
    match CONFIG.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => cx.throw_error(format!(
            "When attempting to obtain config: {}",
            error.to_string()
        )),
    }
}

/// Get the config schema
pub fn schema(cx: FunctionContext) -> JsResult<JsString> {
    let schema = config::schema();
    to_json_or_throw(cx, schema)
}

/// Get the entire global configuration object
pub fn get(mut cx: FunctionContext) -> JsResult<JsString> {
    let config = lock(&mut cx)?;
    to_json(cx, config.clone())
}

/// Set the entire global configuration object
pub fn set(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);

    // Set the config from JSON and write to disk
    let config = &mut *lock(&mut cx)?;
    *config = from_json::<Config>(&mut cx, &json)?;
    if let Err(error) = config.write() {
        return cx.throw_error(error.to_string());
    }

    to_json(cx, config.clone())
}

/// Validate a config
pub fn validate(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);

    let config = from_json::<Config>(&mut cx, &json)?;
    match config.validate() {
        Ok(_) => Ok(cx.boolean(true)),
        Err(error) => {
            cx.throw_error(serde_json::to_string(&error).expect("Unable to convert to JSON"))
        }
    }
}

/// Set a property of the global config
pub fn set_property(mut cx: FunctionContext) -> JsResult<JsString> {
    let pointer = cx.argument::<JsString>(0)?.value(&mut cx);
    let value = cx.argument::<JsString>(1)?.value(&mut cx);

    let config = &mut *lock(&mut cx)?;
    if let Err(error) = config.set(&pointer, &value) {
        return cx.throw_error(error.to_string());
    }

    to_json(cx, config.clone())
}

/// Reset a property of the global config
pub fn reset_property(mut cx: FunctionContext) -> JsResult<JsString> {
    let property = cx.argument::<JsString>(0)?.value(&mut cx);

    let config = &mut *lock(&mut cx)?;
    if let Err(error) = config.reset(&property) {
        return cx.throw_error(error.to_string());
    }

    to_json(cx, config.clone())
}
