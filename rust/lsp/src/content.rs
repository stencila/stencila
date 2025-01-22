//! Handling of custom requests and notifications related to document
//! content.

use std::sync::Arc;

use async_lsp::{
    lsp_types::{notification::Notification, request::Request},
    ClientSocket, ErrorCode, ResponseError,
};

use codecs::{EncodeOptions, Format};
use common::{
    reqwest::Url,
    serde::{Deserialize, Serialize},
    tokio,
    tokio::sync::RwLock,
    tracing,
};
use document::Document;

pub struct SubscribeContent;

impl Request for SubscribeContent {
    const METHOD: &'static str = "stencila/subscribeContent";
    type Params = SubscribeContentParams;
    type Result = String;
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct SubscribeContentParams {
    // The URI of the document for which the content is desired
    pub uri: Url,

    // The format that the content is desired in
    pub format: Format,
}

struct PublishContent;

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct PublishContentParams {
    // The URI of the document for which the content is for
    pub uri: Url,

    // The format that the content is in
    pub format: Format,

    // The content
    pub content: String,
}

impl Notification for PublishContent {
    const METHOD: &'static str = "stencila/publishContent";
    type Params = PublishContentParams;
}

pub async fn subscribe(
    doc: Arc<RwLock<Document>>,
    params: SubscribeContentParams,
    client: ClientSocket,
) -> Result<String, ResponseError> {
    let format = params.format.clone();

    let options = Some(EncodeOptions {
        format: Some(format.clone()),
        ..Default::default()
    });

    let doc = doc.read().await;

    // Start an async task to publish new content to the client
    // when the doc updates
    // TODO: have a way to unsubscribe from these updates when the document
    // preview is closed.
    {
        let mut receiver = doc.watch();
        let options = options.clone();
        let uri = params.uri;
        let format = format.clone();
        tokio::spawn(async move {
            while receiver.changed().await.is_ok() {
                let node = receiver.borrow_and_update().clone();
                match codecs::to_string(&node, options.clone()).await {
                    Ok(content) => {
                        if let Err(error) = client.notify::<PublishContent>(PublishContentParams {
                            uri: uri.clone(),
                            format: format.clone(),
                            content,
                        }) {
                            tracing::error!("While publishing content {error}");
                        };
                    }
                    Err(error) => {
                        let message = format!("When encoding document to {format}: {error}");
                        tracing::error!("{message}");
                        continue;
                    }
                };
            }
        });
    }

    // Return the current content
    match doc.dump(format.clone(), options).await {
        Ok(content) => Ok(content),
        Err(error) => {
            let message = format!("When encoding document to {format}: {error}");
            tracing::error!("{message}");
            Err(ResponseError::new(ErrorCode::INTERNAL_ERROR, message))
        }
    }
}
