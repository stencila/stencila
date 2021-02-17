use neon::prelude::*;

fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /*
    This implements thread safe callbacks but is not quite ready yet
    See
        -  https://docs.rs/neon/0.7.0-napi.3/neon/event/struct.EventQueue.html
        - https://github.com/neon-bindings/neon/pull/622#issuecomment-756763449

    let dispatch = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let queue = cx.queue();

    let node_delegator = Box::new(
        move |method: Method, params: serde_json::Value| -> Result<Node> {
            let _span = tracing::trace_span!("node_delegator");

            // Call the dispatch function the the name of the method and it's
            // parameters as a dictionary
            let this = cx.undefined();
            let method = method.to_string();
            let params = serde_json::to_string(&params)?;
            let args = vec![];
            queue.send(move |mut cx| {
                let callback = dispatch.into_inner(&mut cx);
                match callback.call(&mut cx, this, args) {
                    // Convert the returned JSON string into a `Node` (a `serde_json::Value`)
                    Ok(value) => {
                        println!("OK!");
                        Ok(serde_json::Value::Null)
                    }
                    // Convert any raised Python error into an `anyhow:Error`
                    Err(error) => bail!(error),
                }
            })
        },
    );

    let result = DELEGATOR.set(node_delegator);
    match result {
        Ok(_) => Ok(cx.undefined()),
        Err(_) => return cx.throw_error("Failed to set delegator".to_string())
    }
    */
    Ok(cx.undefined())
}

fn serve(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let url = match cx.argument_opt(1) {
        Some(arg) => Some(arg.downcast::<JsString>().or_throw(&mut cx)?.value()),
        None => None,
    };

    let background = match cx.argument_opt(3) {
        Some(arg) => arg.downcast::<JsBoolean>().or_throw(&mut cx)?.value(),
        None => true,
    };

    match if background {
        stencila::serve::serve_background(url, None)
    } else {
        stencila::serve::serve_blocking(url, None)
    } {
        Ok(_) => Ok(cx.undefined()),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

register_module!(mut cx, {
    cx.export_function("init", init)?;
    cx.export_function("serve", serve)
});
