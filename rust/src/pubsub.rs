use eyre::{bail, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};

pub type Subscriber = fn(topic: String, event: serde_json::Value) -> ();

struct Subscription {
    topic: String,
    subscriber: Subscriber,
}

static SUBSCRIPTIONS: Lazy<Mutex<Vec<Subscription>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Obtain the subscriptions store
fn obtain() -> Result<MutexGuard<'static, Vec<Subscription>>> {
    match SUBSCRIPTIONS.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => bail!(
            "When attempting to obtain subscriptions: {}",
            error.to_string()
        ),
    }
}

/// Subscribe to a topic
pub fn subscribe(topic: &str, subscriber: Subscriber) -> Result<()> {
    let mut subscriptions = obtain()?;
    subscriptions.push(Subscription {
        topic: topic.to_string(),
        subscriber,
    });
    Ok(())
}

/// Publish an event for a topic
pub fn publish(topic: &str, event: serde_json::Value) -> Result<()> {
    let subscriptions = obtain()?;
    for subscription in &*subscriptions {
        if subscription.topic == "*" || subscription.topic == topic {
            (subscription.subscriber)(topic.into(), event.clone())
        }
    }
    Ok(())
}

/// A progress event
///
/// This is the expected structure of events published on the
/// "progress" topic channel. Although all events are simply `serde_json::Value`,
/// this `struct` provides expectations around the shape of those values
/// bot for publishers and subscribers.
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ProgressEvent {
    /// The id of the task that this progress event relates to
    pub id: Option<String>,

    /// The id of the parent task (if any)
    pub parent: Option<String>,

    /// The event message
    pub message: Option<String>,

    /// The current value
    pub current: Option<i64>,

    /// The expected value when complete
    pub expected: Option<i64>,

    // Whether or not the task is complete
    pub complete: bool,
}

/// Publish an event on the "progress" topic channel
pub fn publish_progress(event: ProgressEvent) -> Result<()> {
    return publish("progress", serde_json::to_value(event)?);
}
