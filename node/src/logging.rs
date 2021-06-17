use crate::prelude::*;
use neon::prelude::*;
use stencila::{config::Config, logging};

pub fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let conf = match cx.argument_opt(0) {
        Some(arg) => {
            let json = arg
                .downcast::<JsString, FunctionContext>(&mut cx)
                .or_throw(&mut cx)?
                .value(&mut cx);
            from_json::<Config>(&mut cx, &json)?
        }
        None => crate::config::lock(&mut cx)?.clone(),
    };

    // Do not emit log events to stderr, instead enable pubsub and file handlers
    if let Err(error) = logging::init(false, true, true, &conf.logging) {
        return cx.throw_error(format!(
            "When attempting to initialize logging: {}",
            error.to_string()
        ));
    };
    Ok(cx.undefined())
}

pub fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    logging::test_events();
    Ok(cx.undefined())
}
