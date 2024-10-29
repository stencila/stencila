//! Handling of custom requests and notifications related to
//! the DOM HTML representation of a document

use std::sync::Arc;

use async_lsp::{
    lsp_types::{notification::Notification, request::Request},
    ClientSocket, ErrorCode, ResponseError,
};

use common::{
    reqwest::Url,
    serde::{Deserialize, Serialize},
    tokio::{
        self,
        sync::{mpsc, RwLock},
    },
    tracing,
};
use document::{Document, DomPatch};

pub struct SubscribeDom;

impl Request for SubscribeDom {
    const METHOD: &'static str = "stencila/subscribeDom";
    type Params = SubscribeDomParams;
    type Result = ();
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct SubscribeDomParams {
    // The URI of the document for which the DOM is desired
    pub uri: Url,
}

struct PublishDom;

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct PublishDomParams {
    // The URI of the document for which the DOM patch is for
    pub uri: Url,

    // The DOM patch
    patch: DomPatch,
}

impl Notification for PublishDom {
    const METHOD: &'static str = "stencila/publishDom";
    type Params = PublishDomParams;
}

pub async fn subscribe(
    doc: Arc<RwLock<Document>>,
    params: SubscribeDomParams,
    client: ClientSocket,
) -> Result<(), ResponseError> {
    let (sender, mut receiver) = mpsc::channel(256);

    // Start task to send patches to the client
    tokio::spawn(async move {
        while let Some(patch) = receiver.recv().await {
            if let Err(error) = client.notify::<PublishDom>(PublishDomParams {
                uri: params.uri.clone(),
                patch,
            }) {
                tracing::error!("While publishing DOM patch {error}");
            };
        }
    });

    // Start the DOM syncing task
    doc.read()
        .await
        .sync_dom(Some(sender))
        .await
        .map_err(|error| ResponseError::new(ErrorCode::INTERNAL_ERROR, error.to_string()))
}
