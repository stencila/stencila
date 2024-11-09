//! Handling of custom requests and notifications related to
//! the DOM HTML representation of a document

use std::{collections::HashMap, sync::Arc};

use async_lsp::{
    lsp_types::{notification::Notification, request::Request},
    ClientSocket, ErrorCode, ResponseError,
};

use common::{
    once_cell::sync::Lazy,
    reqwest::Url,
    serde::{Deserialize, Serialize},
    tokio::{
        self,
        sync::{mpsc, Mutex, RwLock},
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
    type Result = (String, String);
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct SubscribeDomParams {
    // The URI of the document for which the DOM is desired
    pub uri: Url,
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

static TASKS: Lazy<Mutex<HashMap<String, JoinHandle<()>>>> = Lazy::new(Mutex::default);

pub async fn subscribe(
    doc: Arc<RwLock<Document>>,
    client: ClientSocket,
) -> Result<(String, String), ResponseError> {
    let (sender, mut receiver) = mpsc::channel(256);

    let subscription_id = Uuid::now_v7().to_string();

    // Start task to send patches to the client
    let sub_id = subscription_id.clone();
    let task = tokio::spawn(async move {
        while let Some(patch) = receiver.recv().await {
            if let Err(error) = client.notify::<PublishDom>(PublishDomParams {
                subscription_id: sub_id.clone(),
                patch,
            }) {
                tracing::error!("While publishing DOM patch {error}");
            };
        }
    });
    TASKS.lock().await.insert(subscription_id.clone(), task);

    let doc = doc.read().await;

    // Get the document theme
    let theme = doc
        .config()
        .await
        .map_err(|error| ResponseError::new(ErrorCode::INTERNAL_ERROR, error.to_string()))?
        .theme
        .unwrap_or_else(|| "default".into());

    // Start the DOM syncing task
    doc.sync_dom(sender)
        .await
        .map_err(|error| ResponseError::new(ErrorCode::INTERNAL_ERROR, error.to_string()))?;

    Ok((subscription_id, theme))
}

pub async fn unsubscribe(subscription_id: String) -> Result<(), ResponseError> {
    if let Some(task) = TASKS.lock().await.remove(&subscription_id) {
        task.abort();
    }

    Ok(())
}
