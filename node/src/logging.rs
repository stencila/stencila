use crate::prelude::*;
use neon::prelude::*;
use stencila::{config::Config, logging};

pub fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);
    let conf = if json.len() > 0 {
        from_json::<Config>(&mut cx, &json)?
    } else {
        crate::config::obtain(&mut cx)?.clone()
    };

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
