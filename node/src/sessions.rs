use crate::prelude::*;
use neon::prelude::*;
use stencila::sessions;

/// Get the module's schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = sessions::schemas();
    to_json_or_throw(cx, schemas)
}
