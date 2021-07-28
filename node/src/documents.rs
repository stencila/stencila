use crate::prelude::*;
use neon::prelude::*;
use std::sync::{Mutex, MutexGuard};
use stencila::{
    documents::{self, Documents},
    once_cell::sync::Lazy,
};

/// A global documents store
static DOCUMENTS: Lazy<Mutex<Documents>> = Lazy::new(|| Mutex::new(Documents::new()));

/// Obtain the documents store
fn obtain(cx: &mut FunctionContext) -> NeonResult<MutexGuard<'static, Documents>> {
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
    let result = RUNTIME.block_on(async { documents.list().await });
    to_json_or_throw(cx, result)
}

/// Create a document
pub fn create(mut cx: FunctionContext) -> JsResult<JsString> {
    let format = cx.argument::<JsString>(0)?.value(&mut cx);
    let format = if format.is_empty() {
        None
    } else {
        Some(format)
    };
    let documents = &mut *obtain(&mut cx)?;
    to_json_or_throw(cx, documents.create(format))
}

/// Open a document
pub fn open(mut cx: FunctionContext) -> JsResult<JsString> {
    let path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let format = cx.argument::<JsString>(1)?.value(&mut cx);
    let format = if format.is_empty() {
        None
    } else {
        Some(format)
    };
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.open(path, format).await });
    to_json_or_throw(cx, result)
}

/// Get a document
pub fn get(mut cx: FunctionContext) -> JsResult<JsString> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);

    let documents = &mut *obtain(&mut cx)?;
    let document = match documents.get(id) {
        Ok(document) => document,
        Err(error) => return cx.throw_error(error.to_string()),
    };
    let result = RUNTIME.block_on(async { document.lock().await.clone() });
    to_json(cx, result)
}

/// Alter a document's properties
pub fn alter(mut cx: FunctionContext) -> JsResult<JsString> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let path = not_empty_or_none(&cx.argument::<JsString>(1)?.value(&mut cx));
    let format = not_empty_or_none(&cx.argument::<JsString>(2)?.value(&mut cx));

    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async {
        match documents.get(id) {
            Ok(document) => {
                let document = &mut *document.lock().await;
                document.alter(path, format)?;
                Ok(document.clone())
            }
            Err(error) => Err(error),
        }
    });
    to_json_or_throw(cx, result)
}

/// Read a document
pub fn read(mut cx: FunctionContext) -> JsResult<JsString> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.read(id).await });
    to_string_or_throw(cx, result)
}

/// Write a document
pub fn write(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let content = cx.argument::<JsString>(1)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.write(id, Some(content)).await });
    to_undefined_or_throw(cx, result)
}

/// Write a document to another path / format
pub fn write_as(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let path = cx.argument::<JsString>(1)?.value(&mut cx);
    let format = cx.argument::<JsString>(2)?.value(&mut cx);
    let format = if format.is_empty() {
        None
    } else {
        Some(format)
    };
    let theme = cx.argument::<JsString>(3)?.value(&mut cx);
    let theme = if theme.is_empty() { None } else { Some(theme) };
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.write_as(id, &path, format, theme).await });
    to_undefined_or_throw(cx, result)
}

/// Dump a document
pub fn dump(mut cx: FunctionContext) -> JsResult<JsString> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let format = cx.argument::<JsString>(1)?.value(&mut cx);
    let format = if format.is_empty() {
        None
    } else {
        Some(format)
    };
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.dump(id, format).await });
    to_string_or_throw(cx, result)
}

/// Load a document
pub fn load(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let content = cx.argument::<JsString>(1)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.load(id, content).await });
    to_undefined_or_throw(cx, result)
}

/// Subscribe to one or more of a document's topics
pub fn subscribe(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let topic = &cx.argument::<JsString>(1)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.subscribe(id, topic).await });
    to_undefined_or_throw(cx, result)
}

/// Unsubscribe from one or more of a document's topics
pub fn unsubscribe(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let topic = &cx.argument::<JsString>(1)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.unsubscribe(id, topic).await });
    to_undefined_or_throw(cx, result)
}

/// Close a document
pub fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let documents = &mut *obtain(&mut cx)?;
    let result = RUNTIME.block_on(async { documents.close(id).await });
    to_undefined_or_throw(cx, result)
}
