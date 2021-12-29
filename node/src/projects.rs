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
    let updates = &cx.argument::<JsString>(1)?.value(&mut cx);
    let updates = from_json::<Project>(&mut cx, updates)?;

    let result = RUNTIME.block_on(async {
        match PROJECTS.get(&path).await {
            Ok(project) => project.lock().await.write(Some(updates)).await,
            Err(error) => Err(error),
        }
    });
    to_undefined_or_throw(cx, result)
}

/// Add a source
pub fn add_source(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let source = &cx.argument::<JsString>(1)?.value(&mut cx);
    let destination = not_empty_or_none(&cx.argument::<JsString>(2)?.value(&mut cx));
    let name = not_empty_or_none(&cx.argument::<JsString>(3)?.value(&mut cx));

    let result = RUNTIME.block_on(async {
        match PROJECTS.get(&path).await {
            Ok(project) => {
                project
                    .lock()
                    .await
                    .add_source(source, destination, name)
                    .await
            }
            Err(error) => Err(error),
        }
    });
    to_json_or_throw(cx, result)
}

/// Remove a source
pub fn remove_source(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let name = &cx.argument::<JsString>(1)?.value(&mut cx);

    let result = RUNTIME.block_on(async {
        match PROJECTS.get(&path).await {
            Ok(project) => project.lock().await.remove_source(name).await,
            Err(error) => Err(error),
        }
    });
    to_json_or_throw(cx, result)
}

/// Import a new or existing source
pub fn import_source(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let name_or_identifier = &cx.argument::<JsString>(1)?.value(&mut cx);
    let destination = not_empty_or_none(&cx.argument::<JsString>(2)?.value(&mut cx));

    let result = RUNTIME.block_on(async {
        match PROJECTS.get(&path).await {
            Ok(project) => {
                project
                    .lock()
                    .await
                    .import_source(name_or_identifier, destination)
                    .await
            }
            Err(error) => Err(error),
        }
    });
    to_json_or_throw(cx, result)
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
