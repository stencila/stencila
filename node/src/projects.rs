use crate::{
    config::{self},
    prelude::*,
};
use neon::prelude::*;
use std::sync::{Mutex, MutexGuard};
use stencila::{
    once_cell::sync::Lazy,
    projects::{Project, Projects},
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

/// Get project schema
pub fn schema(mut cx: FunctionContext) -> JsResult<JsString> {
    let schema = Project::schema();
    Ok(cx.string(schema))
}

/// List projects
pub fn list(mut cx: FunctionContext) -> JsResult<JsString> {
    let projects = &*obtain(&mut cx)?;
    to_json_or_throw(cx, projects.list())
}

/// Open a project
pub fn open(mut cx: FunctionContext) -> JsResult<JsString> {
    let folder = &cx.argument::<JsString>(0)?.value(&mut cx);
    let projects = &mut *obtain(&mut cx)?;
    let config = &*config::obtain(&mut cx)?;
    to_json_or_throw(cx, projects.open(folder, &config.projects))
}

/// Close a project
pub fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let folder = &cx.argument::<JsString>(0)?.value(&mut cx);
    let projects = &mut *obtain(&mut cx)?;
    match projects.close(folder) {
        Ok(_) => Ok(cx.undefined()),
        Err(error) => cx.throw_error(error.to_string()),
    }
}
