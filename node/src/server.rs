use crate::prelude::*;
use neon::prelude::*;
use stencila::serve;

/// Serve a path
pub fn serve(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);

    let result = RUNTIME.block_on(async { serve::serve(path).await });
    to_string_or_throw(cx, result)
}
