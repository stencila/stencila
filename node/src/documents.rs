use crate::prelude::*;
use neon::prelude::*;
use std::sync::{Mutex, MutexGuard};
use stencila::{
    documents::{self, Documents},
    once_cell::sync::Lazy,
};

/// A global documents store
pub static DOCUMENTS: Lazy<Mutex<Documents>> = Lazy::new(|| Mutex::new(Documents::default()));

/// Obtain the documents store
pub fn obtain(cx: &mut FunctionContext) -> NeonResult<MutexGuard<'static, Documents>> {
    match DOCUMENTS.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => cx.throw_error(format!(
            "When attempting on obtain documents: {}",
            error.to_string()
        )),
    }
}

/// Get the module's schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = documents::schemas();
    to_json_or_throw(cx, schemas)
}

/// List documents
pub fn list(mut cx: FunctionContext) -> JsResult<JsString> {
    let documents = &*obtain(&mut cx)?;
    to_json_or_throw(cx, documents.list())
}

/// Open a document
pub fn open(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    to_json_or_throw(cx, documents.open(path))
}

/// Close a document
pub fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    match documents.close(path) {
        Ok(_) => Ok(cx.undefined()),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Read a document
pub fn read(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    match documents.read(path) {
        Ok(content) => Ok(cx.string(content)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Dump a document
pub fn dump(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    match documents.dump(path) {
        Ok(content) => Ok(cx.string(content)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Load a document
pub fn load(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let content = cx.argument::<JsString>(1)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    match documents.load(path, content) {
        Ok(_) => Ok(cx.undefined()),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Write a document
pub fn write(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let content = cx.argument::<JsString>(1)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    match documents.write(path, Some(content)) {
        Ok(_) => Ok(cx.undefined()),
        Err(error) => cx.throw_error(error.to_string()),
    }
}
