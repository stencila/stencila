use crate::prelude::*;
use neon::prelude::*;
use stencila::graphs;

/// Get the module's schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = graphs::schemas();
    to_json_or_throw(cx, schemas)
}
