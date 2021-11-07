use crate::prelude::*;
use neon::prelude::*;

/// Get the module's schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = graph::schemas();
    to_json_or_throw(cx, schemas)
}
