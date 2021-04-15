use crate::prelude::*;
use neon::prelude::*;
use stencila::config::{self, Config};

pub fn read(mut cx: FunctionContext) -> JsResult<JsString> {
    match config::read() {
        Ok(conf) => to_json(cx, conf),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn write(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let json = cx.argument::<JsString>(0)?.value();

    let conf = from_json::<Config>(&mut cx, &json)?;
    match config::write(&conf) {
        Ok(_) => Ok(cx.undefined()),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn validate(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let json = cx.argument::<JsString>(0)?.value();

    let conf = from_json::<Config>(&mut cx, &json)?;
    match config::validate(&conf) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn set(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value();
    let pointer = cx.argument::<JsString>(1)?.value();
    let value = cx.argument::<JsString>(2)?.value();

    let conf = from_json::<Config>(&mut cx, &json)?;
    match config::set(&conf, &pointer, &value) {
        Ok(conf) => to_json(cx, conf),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

pub fn reset(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value();
    let property = cx.argument::<JsString>(1)?.value();

    let conf = from_json::<Config>(&mut cx, &json)?;
    match config::reset(&conf, &property) {
        Ok(conf) => to_json(cx, conf),
        Err(error) => cx.throw_error(error.to_string()),
    }
}
