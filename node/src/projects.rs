use crate::prelude::*;
use neon::prelude::*;
use stencila::projects::{self, Project, PROJECTS};

/// Get schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schema = projects::schemas();
    to_json_or_throw(cx, schema)
}

/// List projects
pub fn list(cx: FunctionContext) -> JsResult<JsString> {
    let result = RUNTIME.block_on(async { PROJECTS.list().await });
    to_json_or_throw(cx, result)
}

/// Open a project
pub fn open(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);

    let result = RUNTIME.block_on(async { PROJECTS.open(Some(path), true).await });
    to_json_or_throw(cx, result)
}

/// Close a project
pub fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);

    to_undefined_or_throw(cx, RUNTIME.block_on(async { PROJECTS.close(path).await }))
}

/// Write a project
pub fn write(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);

    let result = RUNTIME.block_on(async {
        match PROJECTS.get(&path).await {
            Ok(project) => project.lock().await.write().await,
            Err(error) => Err(error),
        }
    });
    to_undefined_or_throw(cx, result)
}

/// Get a project graph in a desired format
pub fn graph(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let format = &cx.argument::<JsString>(1)?.value(&mut cx);

    let result = RUNTIME.block_on(async {
        match PROJECTS.get(&path).await {
            Ok(project) => project.lock().await.graph.to_format(format),
            Err(error) => Err(error),
        }
    });
    to_string_or_throw(cx, result)
}
