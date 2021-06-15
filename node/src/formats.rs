use crate::prelude::*;
use neon::prelude::*;
use stencila::formats;

/// Get the module's schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = formats::schemas();
    to_json_or_throw(cx, schemas)
}

/// Get the hash map of file formats
pub fn formats(cx: FunctionContext) -> JsResult<JsString> {
    let formats = &*formats::FORMATS;
    to_json(cx, formats)
}
