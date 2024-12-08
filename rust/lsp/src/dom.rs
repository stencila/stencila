//! Handling of custom requests and notifications related to
//! the DOM HTML representation of a document

use std::{collections::HashMap, sync::Arc};

use async_lsp::{
    lsp_types::{notification::Notification, request::Request, Uri},
    ClientSocket, ErrorCode, ResponseError,
};

use common::{
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    tokio::{
        self,
        sync::{
            mpsc::{self, Sender},
            Mutex, RwLock,
        },
        task::JoinHandle,
    },
    tracing,
    uuid::Uuid,
};
use document::{Document, DomPatch};

pub struct SubscribeDom;

impl Request for SubscribeDom {
    const METHOD: &'static str = "stencila/subscribeDom";
    type Params = SubscribeDomParams;
    type Result = (String, String, String);
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct SubscribeDomParams {
    // The URI of the document for which the DOM is desired
    pub uri: Uri,
}

pub struct ResetDom;

impl Request for ResetDom {
    const METHOD: &'static str = "stencila/resetDom";
    type Params = ResetDomParams;
    type Result = ();
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ResetDomParams {
    // The id of the subscription
    pub subscription_id: String,
}

pub struct UnsubscribeDom;

impl Request for UnsubscribeDom {
    const METHOD: &'static str = "stencila/unsubscribeDom";
    type Params = UnsubscribeDomParams;
    type Result = ();
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct UnsubscribeDomParams {
    // The id of the subscription
    pub subscription_id: String,
}

struct PublishDom;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PublishDomParams {
    // The id of the subscription
    subscription_id: String,

    // The DOM patch
    patch: DomPatch,
}

impl Notification for PublishDom {
    const METHOD: &'static str = "stencila/publishDom";
    type Params = PublishDomParams;
}

/// A map of subscriptions to document DOMs
#[allow(clippy::type_complexity)]
static SUBSCRIPTIONS: Lazy<Mutex<HashMap<String, (Sender<DomPatch>, JoinHandle<()>)>>> =
    Lazy::new(Mutex::default);

/// Handle a request to subscribe to DOM HTML updates for a document
pub async fn subscribe(
    doc: Arc<RwLock<Document>>,
    client: ClientSocket,
) -> Result<(String, String, String), ResponseError> {
    let (in_sender, in_receiver) = mpsc::channel(256);
    let (out_sender, mut out_receiver) = mpsc::channel(256);

    let subscription_id = Uuid::now_v7().to_string();

    // Start task to send patches to the client
    let sub_id = subscription_id.clone();
    let task = tokio::spawn(async move {
        while let Some(patch) = out_receiver.recv().await {
            if let Err(error) = client.notify::<PublishDom>(PublishDomParams {
                subscription_id: sub_id.clone(),
                patch,
            }) {
                tracing::error!("While publishing DOM patch {error}");
            };
        }
    });

    SUBSCRIPTIONS
        .lock()
        .await
        .insert(subscription_id.clone(), (in_sender, task));

    let doc = doc.read().await;

    // Get the document theme
    let theme = doc
        .config()
        .await
        .map_err(|error| ResponseError::new(ErrorCode::INTERNAL_ERROR, error.to_string()))?
        .theme
        .unwrap_or_else(|| "default".into());

    // Start the DOM syncing task and the initial HTML content
    let html = doc
        .sync_dom(in_receiver, out_sender)
        .await
        .map_err(|error| ResponseError::new(ErrorCode::INTERNAL_ERROR, error.to_string()))?;

    Ok((subscription_id, theme, html))
}

/// Handle a request to reset the DOM HTML for a document
pub async fn reset(subscription_id: String) -> Result<(), ResponseError> {
    if let Some((sender, ..)) = SUBSCRIPTIONS.lock().await.get(&subscription_id) {
        sender
            .send(DomPatch::reset_request())
            .await
            .map_err(|error| ResponseError::new(ErrorCode::INTERNAL_ERROR, error.to_string()))?;
    }

    Ok(())
}

/// Handle a request to unsubscribe from DOM HTML updates for a document
pub async fn unsubscribe(subscription_id: String) -> Result<(), ResponseError> {
    if let Some((.., task)) = SUBSCRIPTIONS.lock().await.remove(&subscription_id) {
        task.abort();
    }

    Ok(())
}
