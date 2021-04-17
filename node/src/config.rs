use crate::prelude::*;
use neon::prelude::*;
use std::sync::{Mutex, MutexGuard};
use stencila::{
    config::{self, Config},
    once_cell::sync::Lazy,
};

// A config object needs to be read on startup and then passed to various
// functions in other modules on each invocation.
// We want to avoid exposing that implementation detail in these bindings
// so have this global mutable 😱 configuration object that gets
// read when the module is loaded, updated in the functions below
// and then passed on to other functions
pub static CONFIG: Lazy<Mutex<Config>> =
    Lazy::new(|| Mutex::new(config::read().expect("Unable to read config")));

pub fn obtain(cx: &mut FunctionContext) -> NeonResult<MutexGuard<'static, Config>> {
    match CONFIG.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => cx.throw_error(format!(
            "When attempting on obtain config: {}",
            error.to_string()
        )),
    }
}

fn save_then_to_json(cx: FunctionContext, conf: Config) -> JsResult<JsString> {
    let mut guard = CONFIG.lock().expect("Unable to lock config");
    *guard = conf;
    to_json(cx, guard.to_owned())
}

pub fn read(mut cx: FunctionContext) -> JsResult<JsString> {
    match config::read() {
        Ok(conf) => save_then_to_json(cx, conf),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn write(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);

    let conf = from_json::<Config>(&mut cx, &json)?;
    match config::write(&conf) {
        Ok(_) => Ok(cx.undefined()),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn validate(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);

    let conf = from_json::<Config>(&mut cx, &json)?;
    match config::validate(&conf) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn set(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);
    let pointer = cx.argument::<JsString>(1)?.value(&mut cx);
    let value = cx.argument::<JsString>(2)?.value(&mut cx);

    let conf = from_json::<Config>(&mut cx, &json)?;
    match config::set(&conf, &pointer, &value) {
        Ok(conf) => save_then_to_json(cx, conf),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn reset(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);
    let property = cx.argument::<JsString>(1)?.value(&mut cx);

    let conf = from_json::<Config>(&mut cx, &json)?;
    match config::reset(&conf, &property) {
        Ok(conf) => save_then_to_json(cx, conf),
        Err(error) => cx.throw_error(error.to_string()),
    }
}
