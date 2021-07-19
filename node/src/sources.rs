use crate::prelude::*;
use neon::prelude::*;
use stencila::sources;

/// Get the module's schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = sources::schemas();
    to_json_or_throw(cx, schemas)
}
