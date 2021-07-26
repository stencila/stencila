use crate::prelude::*;
use neon::prelude::*;
use std::sync::{Mutex, MutexGuard};
use stencila::{
    once_cell::sync::Lazy,
    projects::{self, Project, Projects},
};

/// A global projects store
pub static PROJECTS: Lazy<Mutex<Projects>> = Lazy::new(|| Mutex::new(Projects::default()));

/// Lock the projects store
pub fn lock(cx: &mut FunctionContext) -> NeonResult<MutexGuard<'static, Projects>> {
    match PROJECTS.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => cx.throw_error(format!(
            "When attempting to lock projects: {}",
            error.to_string()
        )),
    }
}

/// Get schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schema = projects::schemas();
    to_json_or_throw(cx, schema)
}

/// List projects
pub fn list(mut cx: FunctionContext) -> JsResult<JsString> {
    let projects = &*lock(&mut cx)?;
    let result = RUNTIME.block_on(async { projects.list().await });
    to_json_or_throw(cx, result)
}

/// Open a project
pub fn open(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let projects = &mut *lock(&mut cx)?;
    let result = RUNTIME.block_on(async { projects.open(Some(path), true).await });
    to_json_or_throw(cx, result)
}

/// Close a project
pub fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let projects = &mut *lock(&mut cx)?;
    to_undefined_or_throw(cx, projects.close(path))
}

/// Write a project
pub fn write(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let updates = &cx.argument::<JsString>(1)?.value(&mut cx);
    let updates = from_json::<Project>(&mut cx, &updates)?;

    let projects = &mut *lock(&mut cx)?;
    let result = RUNTIME.block_on(async {
        match projects.get(&path) {
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

    let projects = &mut *lock(&mut cx)?;
    let result = RUNTIME.block_on(async {
        match projects.get(&path) {
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

    let projects = &mut *lock(&mut cx)?;
    let result = RUNTIME.block_on(async {
        match projects.get(&path) {
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

    let projects = &mut *lock(&mut cx)?;
    let result = RUNTIME.block_on(async {
        match projects.get(&path) {
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
