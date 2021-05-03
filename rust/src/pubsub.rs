use eyre::{bail, Result};
use once_cell::sync::Lazy;
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

pub fn publish(topic: &str, event: serde_json::Value) -> Result<()> {
    let subscriptions = obtain()?;
    for subscription in &*subscriptions {
        if subscription.topic == "*" {
            (subscription.subscriber)(topic.into(), event.clone())
        }
    }
    Ok(())
}
