use crate::{
    config::{self},
    prelude::*,
};
use neon::prelude::*;
use std::sync::{Mutex, MutexGuard};
use stencila::{
    once_cell::sync::Lazy,
    projects::{self, Project, Projects},
};

/// A global projects store
pub static PROJECTS: Lazy<Mutex<Projects>> = Lazy::new(|| Mutex::new(Projects::default()));

/// Obtain the projects store
pub fn obtain(cx: &mut FunctionContext) -> NeonResult<MutexGuard<'static, Projects>> {
    match PROJECTS.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => cx.throw_error(format!(
            "When attempting on obtain projects: {}",
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
    let projects = &*obtain(&mut cx)?;
    to_json_or_throw(cx, projects.list())
}

/// Open a project
pub fn open(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let projects = &mut *obtain(&mut cx)?;
    let config = &*config::obtain(&mut cx)?;
    to_json_or_throw(cx, projects.open(path, &config.projects, true))
}

/// Write a project
pub fn write(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let updates = &cx.argument::<JsString>(1)?.value(&mut cx);
    let updates = from_json::<Project>(&mut cx, &updates)?;
    let projects = &mut *obtain(&mut cx)?;
    to_undefined_or_throw(cx, projects.write(path, Some(updates)))
}

/// Close a project
pub fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let projects = &mut *obtain(&mut cx)?;
    to_undefined_or_throw(cx, projects.close(path))
}
