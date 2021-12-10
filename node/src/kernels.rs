use crate::prelude::*;
use neon::prelude::*;
use stencila::kernels;

/// Get the available kernels
pub fn languages(cx: FunctionContext) -> JsResult<JsString> {
    let result = RUNTIME.block_on(async { kernels::languages().await });
    to_json_or_throw(cx, result)
}
