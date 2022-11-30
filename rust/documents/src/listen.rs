//! Functions related to document reactivity

use common::{eyre::Result, tracing};
use events::{subscribe, unsubscribe, SubscriptionId, SubscriptionTopic};

use crate::{
    document::{
        Document, DocumentEventListener, DocumentEventListeners, DocumentEventReceiver,
        DocumentEventSender,
    },
    messages::{send_any_request, DocumentRequestSenders},
};

impl Document {
    /// Listen to an event topic
    pub async fn listen(
        event_sender: &DocumentEventSender,
        event_listeners: &DocumentEventListeners,
        listener_id: &str,
        event_topic: &str,
        event_listener: DocumentEventListener,
    ) -> Result<()> {
        let (event_listeners, event_subscriptions) = &mut *event_listeners.write().await;

        Self::listen_impl(
            event_sender,
            event_listeners,
            event_subscriptions,
            listener_id,
            event_topic,
            event_listener,
        )?;

        Ok(())
    }

    /// Listen to many event topics
    pub async fn listen_many(
        event_sender: &DocumentEventSender,
        event_listeners: &DocumentEventListeners,
        new_listeners: Vec<(String, String, DocumentEventListener)>,
    ) -> Result<()> {
        let (event_listeners, event_subscriptions) = &mut *event_listeners.write().await;

        for (listener_id, event_topic, event_listener) in new_listeners {
            Self::listen_impl(
                event_sender,
                event_listeners,
                event_subscriptions,
                &listener_id,
                &event_topic,
                event_listener,
            )?;
        }

        Ok(())
    }

    /// Listen to an event topic
    fn listen_impl(
        event_sender: &DocumentEventSender,
        event_listeners: &mut Vec<(String, String, DocumentEventListener)>,
        event_subscriptions: &mut Vec<(SubscriptionTopic, SubscriptionId)>,
        listener_id: &str,
        event_topic: &str,
        event_listener: DocumentEventListener,
    ) -> Result<()> {
        let already_registered =
            event_listeners
                .iter()
                .any(|(existing_listener_id, existing_topic, ..)| {
                    listener_id == existing_listener_id && event_topic == existing_topic
                });
        if already_registered {
            return Ok(());
        }

        tracing::debug!("Listening to event topic `{}`", event_topic);

        // Subscribe to the event topic if there are no listeners already subscribed to the
        // the topic
        if !event_subscriptions
            .iter()
            .any(|(topic, ..)| topic == event_topic)
        {
            let id = subscribe(
                event_topic,
                events::Subscriber::UnboundedSender(event_sender.clone()),
            )?;
            event_subscriptions.push((event_topic.to_string(), id));
        }

        // Store the event listener
        event_listeners.push((
            listener_id.to_string(),
            event_topic.to_string(),
            event_listener,
        ));

        Ok(())
    }

    /// Stop listening to an event topic
    pub async fn unlisten(
        event_listeners: &DocumentEventListeners,
        event_topic: &str,
    ) -> Result<()> {
        tracing::debug!("Un-listening from event topic `{}`", event_topic);

        let (event_listeners, event_subscriptions) = &mut *event_listeners.write().await;

        // Unsubscribe from the topic
        for (topic, id) in event_subscriptions.iter() {
            if topic == event_topic {
                unsubscribe(id)?;
            }
        }
        event_subscriptions.retain(|(topic, ..)| topic != event_topic);

        // Remove _all_ listeners listening to the topic
        event_listeners.retain(|(topic, ..)| topic != event_topic);

        Ok(())
    }

    /// A task to listen for events
    pub async fn listen_task(
        document_id: &str,
        event_receiver: &mut DocumentEventReceiver,
        event_listeners: &DocumentEventListeners,
        request_senders: &DocumentRequestSenders,
    ) {
        while let Some((event_topic, event_detail)) = event_receiver.recv().await {
            tracing::debug!(
                "Listen task for document `{}` received an event for topic `{}`",
                document_id,
                event_topic
            );

            let (event_listeners, ..) = &*event_listeners.read().await;
            for (listener_id, listener_topic, listener) in event_listeners.iter() {
                if event_topic.starts_with(listener_topic) {
                    let request = listener(&event_topic, event_detail.clone());
                    tracing::debug!(
                        "Sending request to `{}` document `{}` for listener `{}`",
                        request.as_ref(),
                        document_id,
                        listener_id
                    );
                    send_any_request(request_senders, request).await;
                }
            }
        }
    }
}
