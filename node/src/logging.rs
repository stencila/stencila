use neon::prelude::*;
use stencila::logging::{init_publish, test_events};

pub fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    if let Err(error) = init_publish(crate::subscriptions::publish_topic_data) {
        return cx.throw_error(format!(
            "When attempting to initialize logging: {}",
            error.to_string()
        ));
    };
    Ok(cx.undefined())
}

pub fn test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    test_events();
    Ok(cx.undefined())
}
