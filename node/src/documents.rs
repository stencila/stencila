use crate::prelude::*;
use neon::prelude::*;
use stencila::documents::{self, DOCUMENTS};

/// Get the module's schemas
pub fn schemas(cx: FunctionContext) -> JsResult<JsString> {
    let schemas = documents::schemas();
    to_json_or_throw(cx, schemas)
}

/// List documents
pub fn list(mut cx: FunctionContext) -> JsResult<JsString> {
    let result = RUNTIME.block_on(async { DOCUMENTS.list().await });
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

    let result = RUNTIME.block_on(async { DOCUMENTS.create(format).await });
    to_json_or_throw(cx, result)
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

    let result = RUNTIME.block_on(async { DOCUMENTS.open(path, format).await });
    to_json_or_throw(cx, result)
}

/// Get a document
pub fn get(mut cx: FunctionContext) -> JsResult<JsString> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => {
                let document = &mut *document.lock().await;
                Ok(document.clone())
            }
            Err(error) => return Err(error),
        }
    });
    to_json_or_throw(cx, result)
}

/// Alter a document's properties
pub fn alter(mut cx: FunctionContext) -> JsResult<JsString> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let path = not_empty_or_none(&cx.argument::<JsString>(1)?.value(&mut cx));
    let format = not_empty_or_none(&cx.argument::<JsString>(2)?.value(&mut cx));

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => {
                let document = &mut *document.lock().await;
                document.alter(path, format).await?;
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

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => document.lock().await.read().await,
            Err(error) => Err(error),
        }
    });
    to_string_or_throw(cx, result)
}

/// Write a document
pub fn write(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let content = cx.argument::<JsString>(1)?.value(&mut cx);

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => document.lock().await.write(Some(content), None).await,
            Err(error) => Err(error),
        }
    });
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

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => document.lock().await.write_as(path, format, theme).await,
            Err(error) => Err(error),
        }
    });
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

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => document.lock().await.dump(format).await,
            Err(error) => Err(error),
        }
    });
    to_string_or_throw(cx, result)
}

/// Load a document
pub fn load(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let content = cx.argument::<JsString>(1)?.value(&mut cx);

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => document.lock().await.load(content, None).await,
            Err(error) => Err(error),
        }
    });
    to_undefined_or_throw(cx, result)
}

/// Subscribe to one or more of a document's topics
pub fn subscribe(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let topic = &cx.argument::<JsString>(1)?.value(&mut cx);

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => document.lock().await.subscribe(topic),
            Err(error) => Err(error),
        }
    });
    to_undefined_or_throw(cx, result)
}

/// Unsubscribe from one or more of a document's topics
pub fn unsubscribe(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);
    let topic = &cx.argument::<JsString>(1)?.value(&mut cx);

    let result = RUNTIME.block_on(async {
        match DOCUMENTS.get(id).await {
            Ok(document) => document.lock().await.unsubscribe(topic),
            Err(error) => Err(error),
        }
    });
    to_undefined_or_throw(cx, result)
}

/// Close a document
pub fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = &cx.argument::<JsString>(0)?.value(&mut cx);

    let result = RUNTIME.block_on(async { DOCUMENTS.close(id).await });
    to_undefined_or_throw(cx, result)
}
