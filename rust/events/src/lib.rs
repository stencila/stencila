use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use common::{
    eyre::{bail, Result},
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    serde_json,
    tokio::{
        self, signal,
        sync::{mpsc, Mutex as AsyncMutex},
    },
    tracing,
};
use uuids::uuid_family;

/// An event updating progress of some task
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
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

pub type Message = (String, serde_json::Value);

uuid_family!(SubscriptionId, "su");

pub enum Subscriber {
    Function(fn(topic: String, event: serde_json::Value) -> ()),
    Sender(mpsc::Sender<Message>),
    UnboundedSender(mpsc::UnboundedSender<Message>),
}

struct Subscription {
    /// The topic that is subscribed to
    topic: String,

    /// The subscriber that is sent an event
    subscriber: Subscriber,
}

/// The global subscriptions store
static SUBSCRIPTIONS: Lazy<Mutex<HashMap<SubscriptionId, Subscription>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Obtain the subscriptions store
fn obtain() -> Result<MutexGuard<'static, HashMap<SubscriptionId, Subscription>>> {
    // Use `lock`, not `try_lock`, which means this thread may block waiting
    // a little for the subscriptions to become available.
    match SUBSCRIPTIONS.lock() {
        Ok(guard) => Ok(guard),
        Err(error) => bail!("While attempting to obtain subscriptions: {}", error),
    }
}

/// Subscribe to a topic
#[allow(clippy::let_unit_value)]
pub fn subscribe(topic: &str, subscriber: Subscriber) -> Result<SubscriptionId> {
    tracing::trace!("Subscribing to topic `{}`", topic);

    // Ensure the interrupt listener is started
    if topic == "interrupt" {
        let _ = *INTERRUPT_LISTENER;
    }

    match obtain() {
        Ok(mut subscriptions) => {
            let id = SubscriptionId::new();
            subscriptions.insert(
                id.clone(),
                Subscription {
                    topic: topic.to_string(),
                    subscriber,
                },
            );
            Ok(id)
        }
        Err(error) => {
            bail!("Unable to subscribe: {}", error.to_string())
        }
    }
}

/// Unsubscribe
pub fn unsubscribe(subscription_id: &SubscriptionId) -> Result<()> {
    tracing::trace!("Unsubscribing subscription `{}`", subscription_id);

    match obtain() {
        Ok(mut subscriptions) => {
            subscriptions.remove(subscription_id);
            Ok(())
        }
        Err(error) => {
            bail!("Unable to unsubscribe: {}", error.to_string())
        }
    }
}

/// Publish an event for a topic
///
/// Publishing an event should be treated as 'fire-and-forget'.
/// This function does not return an `Err` if it fails but will
/// log an error (if not already attempting to publish to logging channel).
pub fn publish<Event>(topic: &str, event: Event)
where
    Event: Serialize,
{
    if topic != "logging" {
        tracing::trace!("Publishing event for topic `{}`", topic);
    }

    match obtain() {
        Ok(subscriptions) => {
            for subscription in subscriptions.values() {
                if subscription.topic == "*"
                    || subscription.topic == topic
                    || topic.starts_with(&subscription.topic)
                {
                    let value = serde_json::to_value(&event).unwrap_or(serde_json::Value::Null);
                    match &subscription.subscriber {
                        Subscriber::Function(function) => {
                            function(topic.into(), value);
                        }
                        Subscriber::Sender(sender) => {
                            let sender = sender.clone();
                            let topic = topic.to_string();
                            tokio::spawn(async move {
                                if let Err(error) = sender.send((topic, value)).await {
                                    tracing::error!(
                                        "Error sending event on bounded channel: {}",
                                        error
                                    );
                                }
                            });
                        }
                        Subscriber::UnboundedSender(sender) => {
                            if let Err(error) = sender.send((topic.into(), value)) {
                                tracing::error!(
                                    "Error sending event on unbounded channel: {}",
                                    error
                                );
                            }
                        }
                    }
                }
            }
        }
        Err(error) => {
            // Do not log error if the topic is logging since that could lead to recursion
            if topic != "logging" {
                tracing::error!("Unable to publish event: {}", error)
            }
        }
    }
}

/// The list of subscribers to the interrupt topic
static INTERRUPT_SUBSCRIBERS: Lazy<AsyncMutex<Vec<mpsc::Sender<()>>>> =
    Lazy::new(|| AsyncMutex::new(Vec::new()));

/// A lazily started Ctrl-C listener.
static INTERRUPT_LISTENER: Lazy<()> = Lazy::new(|| {
    tokio::spawn(async move {
        if let Ok(..) = signal::ctrl_c().await {
            publish("interrupt", true);

            let subscribers = INTERRUPT_SUBSCRIBERS.lock().await;
            for subscriber in subscribers.iter() {
                if let Err(..) = subscriber.send(()).await {
                    // Ignore error (in the case that receiver has already closed)
                }
            }
        }
    });
});

/// Subscribe to the interrupt topic with a subscribing bounded channel sender
///
/// This is an alternative to using `subscribe("interrupt")` that is more convenient
/// when you want to be able to swap the interrupt channel for some other cancellation
/// channel that is bounded and without a message type (ie. `mpsc::Sender<()>` rather than `mpsc::Sender<Message>`)
/// and you don't need to unsubscribe.
#[allow(clippy::let_unit_value)]
pub async fn subscribe_to_interrupt(subscriber: mpsc::Sender<()>) {
    // Ensure the interrupt listener is started
    let _ = *INTERRUPT_LISTENER;

    // Add the subscriber
    let mut subscribers = INTERRUPT_SUBSCRIBERS.lock().await;
    subscribers.push(subscriber);
}
