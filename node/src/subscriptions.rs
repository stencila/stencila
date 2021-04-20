use neon::prelude::*;
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};
use stencila::{
    once_cell::sync::{Lazy, OnceCell},
    serde_json,
};

use crate::prelude::from_json;

static QUEUE: OnceCell<EventQueue> = OnceCell::new();

static SUBSCRIPTIONS: Lazy<Mutex<HashMap<String, Root<JsFunction>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Obtain the subscriptions store
pub fn obtain(
    cx: &mut FunctionContext,
) -> NeonResult<MutexGuard<'static, HashMap<String, Root<JsFunction>>>> {
    match SUBSCRIPTIONS.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => cx.throw_error(format!(
            "When attempting to obtain subscriptions: {}",
            error.to_string()
        )),
    }
}

/// Subscribe to a topic
pub fn subscribe(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let topic = cx.argument::<JsString>(0)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);

    let queue = cx.queue();
    if let Err(_) = QUEUE.set(queue) {
        // Ignore because it just means queue was already set
    }

    let mut subscriptions = obtain(&mut cx)?;
    subscriptions.insert(topic, callback);

    Ok(cx.undefined())
}

/// Unsubscribe from a topic
pub fn unsubscribe(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let topic = cx.argument::<JsString>(0)?.value(&mut cx);

    let mut subscriptions = obtain(&mut cx)?;
    subscriptions.remove(&topic);

    Ok(cx.undefined())
}

/// Publish data for a topic
pub fn publish(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let topic = cx.argument::<JsString>(0)?.value(&mut cx);
    let json = cx.argument::<JsString>(1)?.value(&mut cx);

    publish_topic_data(topic, from_json::<serde_json::Value>(&mut cx, &json)?);

    Ok(cx.undefined())
}

/// Publish data for a topic
///
/// This function is intended to be called by Rust to send data to
/// Node.js subscribers.
fn publish_topic_data(topic: String, data: serde_json::Value) -> () {
    // If the queue is not sent then it means that there are
    // no subscribers and so no need to do anything
    if let Some(queue) = QUEUE.get() {
        queue.send(move |mut cx| {
            if let Some(func) = SUBSCRIPTIONS
                .lock()
                .expect("Unable to obtain subscriptions lock")
                .get(&topic)
            {
                let callback = func.to_inner(&mut cx);
                let this = cx.undefined();
                let json = serde_json::to_string(&data).expect("Unable to convert to JSON");
                let args = vec![cx.string(topic), cx.string(json)];
                callback.call(&mut cx, this, args)?;
            }
            Ok(())
        });
    }
}
