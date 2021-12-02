use crate::prelude::*;
use neon::prelude::*;
use stencila::kernels;

/// Get the available kernels
pub fn available(cx: FunctionContext) -> JsResult<JsString> {
    let result = RUNTIME.block_on(async { kernels::available().await });
    to_json_or_throw(cx, result)
}
