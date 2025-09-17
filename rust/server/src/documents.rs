use std::{
    collections::HashMap,
    path::{Component, PathBuf},
    sync::Arc,
};

use axum::{
    Json, Router,
    body::Body,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderName, HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use eyre::{Result, eyre};
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use itertools::Itertools;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    self,
    fs::read,
    sync::{
        RwLock,
        mpsc::{Receiver, Sender, channel},
    },
};
use uuid::Uuid;

use stencila_codecs::{DecodeOptions, EncodeOptions};
use stencila_document::{Document, SyncDirection};
use stencila_format::Format;

use crate::{
    errors::InternalError,
    server::{STENCILA_VERSION, ServerState},
};

/// A store of documents
#[derive(Debug, Default)]
pub(crate) struct Documents {
    /// A mapping between a file system path and the id of the trunk
    /// [`Document`] instance for that path
    paths: RwLock<HashMap<PathBuf, Uuid>>,

    /// A mapping of document ids to [`Document`]s
    docs: RwLock<HashMap<Uuid, Arc<Document>>>,
}

impl Documents {
    /// Get a document by [`Uuid`]
    async fn by_uuid(&self, uuid: &Uuid) -> Result<Arc<Document>> {
        let doc = self
            .docs
            .read()
            .await
            .get(uuid)
            .ok_or_else(|| eyre!("No doc with UUID `{uuid}`"))?
            .clone();

        Ok(doc)
    }

    /// Get a document by UUID string
    async fn by_uuid_string(&self, uuid: &str) -> Result<Arc<Document>> {
        self.by_uuid(&Uuid::parse_str(uuid)?).await
    }

    /// Get a document by path
    ///
    /// At present this always returns the trunk document for the path.
    /// In the future, based on arguments and/or the user's permissions on the
    /// document, will return a branch or a twig document.
    async fn by_path(
        &self,
        path: &std::path::Path,
        sync: Option<SyncDirection>,
    ) -> Result<(Uuid, Arc<Document>)> {
        {
            // In block to ensure lock is dropped when no longer needed
            let paths = self.paths.read().await;
            if let Some(uuid) = paths.get(path) {
                return Ok((*uuid, self.by_uuid(uuid).await?));
            }
        }

        // Open the document
        let doc = if let Some(direction) = sync {
            Document::synced(path, direction).await?
        } else {
            Document::open(path, None).await?
        };

        // Compile the document (so math, headings list, etc can be properly encoded to HTML)
        doc.compile().await?;

        let uuid = Uuid::new_v4();

        self.paths.write().await.insert(path.to_path_buf(), uuid);
        self.docs.write().await.insert(uuid, Arc::new(doc));

        let doc = self.by_uuid(&uuid).await?;

        Ok((uuid, doc))
    }

    /// Close a document by [`Uuid`]
    #[allow(unused)]
    async fn close(&self, uuid: &Uuid) -> Result<()> {
        self.docs.write().await.remove(uuid);

        // TODO: When there are multiple docs for a path this will need to be revised.
        self.paths
            .write()
            .await
            .retain(|_, entry_uuid| entry_uuid != uuid);

        Ok(())
    }
}

/// Create a router for document routes
pub fn router() -> Router<ServerState> {
    Router::new().route("/{id}/websocket", get(websocket_for_document))
}

/// Serve the root path
///
/// Only exists because we can't route to `serve_path` directly
/// for an empty route.
#[tracing::instrument(skip_all)]
pub async fn serve_root(
    state: State<ServerState>,
    query: Query<HashMap<String, String>>,
) -> Result<Response, InternalError> {
    serve_path(state, Path(String::new()), query).await
}

/// Serve a document
#[tracing::instrument(skip(docs))]
pub async fn serve_path(
    State(ServerState {
        dir,
        raw,
        source,
        docs,
        sync,
        ..
    }): State<ServerState>,
    Path(path): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, InternalError> {
    // Path should be within served `dir`
    let path = dir.join(path);

    // Check for attempts at directory traversal and to access private file or directory.
    if path.components().any(|component| {
        matches!(component, Component::ParentDir)
            || component.as_os_str().to_string_lossy().starts_with('_')
    }) {
        return Ok(not_found());
    }

    // Resolve the URL path into a filesystem path
    let path = Document::resolve_file(&path).map_err(InternalError::new)?;

    // Return early if no path resolved
    let Some(path) = path else {
        return Ok(not_found());
    };

    // Return early if the path is a media file
    let format = Format::from_path(&path);
    if format.is_media() {
        let bytes = read(&path).await.map_err(InternalError::new)?;
        let content_type = mime_guess::from_path(path).first_or_octet_stream();

        return Response::builder()
            .header(CONTENT_TYPE, content_type.essence_str())
            .body(Body::from(bytes))
            .map_err(InternalError::new);
    }

    // Get the document for the path
    let (uuid, doc) = docs
        .by_path(&path, sync)
        .await
        .map_err(InternalError::new)?;
    let config = doc.config().await.map_err(InternalError::new)?;

    // Early-returned response for raw
    if query.contains_key("~raw") {
        // If raw is enabled early return a response with the content of the file
        if !raw {
            return Ok(StatusCode::FORBIDDEN.into_response());
        }

        let bytes = read(&path).await.map_err(InternalError::new)?;
        let content_type = mime_guess::from_path(path).first_or_octet_stream();

        return Response::builder()
            .header(CONTENT_TYPE, content_type.essence_str())
            .body(Body::from(bytes))
            .map_err(InternalError::new);
    }

    let view = query
        .get("~view")
        .map_or("dynamic", |value: &String| value.as_ref());

    let theme = query
        .get("~theme")
        .map(|value: &String| value.as_str())
        .or(config.theme.as_deref())
        .unwrap_or("default");

    // Generate the body of the HTML
    // Note that for dynamic views, when WebSocket connection is made, a "reset patch" will be sent with the same
    // root HTML. This is somewhat redundant, but is necessary, given that we need to have a known version of the
    // HTML as the basis for patching. We could skip including the HTML here (we used to) but then that is unsafe
    // if there are Websocket issues (the page would be blank).
    let root_type = doc.inspect(|root| root.node_type()).await;
    let root_html = doc
        .dump(Format::Dom, None)
        .await
        .map_err(InternalError::new)?;

    // The URL prefix for Stencila's web distribution
    let web = if cfg!(debug_assertions) {
        "/~static/dev".to_string()
    } else {
        ["/~static/", STENCILA_VERSION].concat()
    };

    let html = format!(
        r#"<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8"/>
        <title>Stencila</title>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="icon" type="image/png" href="{web}/images/favicon.png">
        <link rel="preconnect" href="https://fonts.googleapis.com">
        <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap" rel="stylesheet">
        <link rel="stylesheet" type="text/css" href="{web}/themes/{theme}.css">
        <link rel="stylesheet" type="text/css" href="{web}/views/{view}.css">
        <script type="module" src="{web}/views/{view}.js"></script>
    </head>
    <body>
        <stencila-{view}-view view={view} doc={uuid} type={root_type}>
            {root_html}
        </stencila-{view}-view>
    </body>
</html>"#
    );

    // Build the response
    let response = (|| -> eyre::Result<Response> {
        let mut response = Response::builder()
        // TODO set the content type header
        //.header(CONTENT_TYPE, format.media_type())
        ;

        if source && let Ok(path) = path.strip_prefix(&dir) {
            response = response.header(
                HeaderName::try_from("SourceMap")?,
                HeaderValue::from_str(&path.to_string_lossy())?,
            );
        }

        Ok(response.body(Body::from(html))?)
    })()
    .map_err(InternalError::new)?;

    Ok(response)
}

/// Open a document and return its id
#[tracing::instrument(skip(docs))]
async fn open_document(
    State(ServerState {
        dir,
        raw,
        source,
        docs,
        sync,
        ..
    }): State<ServerState>,
    Path(path): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, InternalError> {
    // Path should be within served `dir`
    let path = dir.join(path);

    // Check for attempts at directory traversal and to access private file or directory.
    if path.components().any(|component| {
        matches!(component, Component::ParentDir)
            || component.as_os_str().to_string_lossy().starts_with('_')
    }) {
        return Ok(not_found());
    }

    // Resolve the URL path into a filesystem path
    let path = Document::resolve_file(&path).map_err(InternalError::new)?;

    // Return early if no path resolved
    let Some(path) = path else {
        return Ok(not_found());
    };

    // Get the document for the path
    let (uuid, ..) = docs
        .by_path(&path, sync)
        .await
        .map_err(InternalError::new)?;

    #[derive(Serialize)]
    struct OpenResponse {
        id: String,
    }

    Ok(Json(OpenResponse {
        id: uuid.to_string(),
    })
    .into_response())
}

/// Handle a WebSocket upgrade request
async fn websocket_for_document(
    State(ServerState {
        dir, docs, sync, ..
    }): State<ServerState>,
    ws: WebSocketUpgrade,
    Path(uuid): Path<String>,
) -> Result<Response, InternalError> {
    let Ok(doc) = docs.by_uuid_string(&uuid).await else {
        return Ok((StatusCode::BAD_REQUEST, "Invalid document id").into_response());
    };

    // TODO: Change the allowed protocols based on the users permissions
    let mut protocols = vec![
        "read.dom.stencila.org".to_string(),
        "read.debug.stencila.org".to_string(),
        "read.object.stencila.org".to_string(),
    ];

    // Protocols only permitted if sync direction includes `Out`
    if matches!(sync, Some(SyncDirection::Out | SyncDirection::InOut)) {
        // Note that there is no `read.directory` protocol: directories
        // are read using `read.object` protocol
        protocols.push("write.directory.stencila.org".to_string())
    }

    for format in [
        // TODO: define this list of string formats better
        Format::Dom,
        Format::Html,
        Format::Jats,
        Format::Json,
        Format::Json5,
        Format::JsonLd,
        Format::Markdown,
        Format::Smd,
        Format::Qmd,
        Format::Myst,
        Format::Yaml,
    ] {
        protocols.push(format!("read.{format}.stencila.org"));
        protocols.push(format!("write.{format}.stencila.org"));
    }

    for access in [
        "comment", "suggest", "input", "code", "prose", "write", "admin",
    ] {
        protocols.push(format!("{access}.nodes.stencila.org"));
    }

    // During development allow `write.dom` protocol so that source view
    // can be used for viewing DOM HTML
    #[cfg(debug_assertions)]
    {
        protocols.push("write.dom.stencila.org".to_string());
    }

    let response = ws
        .protocols(protocols)
        .on_upgrade(move |ws| websocket_handler(ws, doc, dir));

    Ok(response)
}

/// Handle a WebSocket connection
#[tracing::instrument(skip(ws, doc))]
async fn websocket_handler(mut ws: WebSocket, doc: Arc<Document>, dir: PathBuf) {
    tracing::trace!("WebSocket connection");

    let Some(protocol) = ws
        .protocol()
        .and_then(|header| header.to_str().ok())
        .map(String::from)
    else {
        tracing::debug!("No WebSocket subprotocol");
        ws.close().await.ok();
        return;
    };

    let Some(protocol) = protocol.strip_suffix(".stencila.org") else {
        tracing::debug!("Unknown WebSocket subprotocol: {protocol}");
        ws.close().await.ok();
        return;
    };

    let Some((capability, format)) = protocol.split('.').collect_tuple() else {
        tracing::debug!("Invalid WebSocket subprotocol: {protocol}");
        ws.close().await.ok();
        return;
    };

    if format == "nodes" {
        websocket_nodes_protocol(ws, doc, capability).await;
    } else if format == "object" {
        websocket_object_protocol(ws, doc, capability).await;
    } else if format == "directory" {
        websocket_directory_protocol(ws, doc, capability, dir).await;
    } else if format == "dom" {
        websocket_dom_protocol(ws, doc).await;
    } else {
        websocket_format_protocol(ws, doc, capability, format).await;
    }
}

/// Handle a WebSocket connection using the "nodes" protocol
#[tracing::instrument(skip(_ws, _doc))]
async fn websocket_nodes_protocol(_ws: WebSocket, _doc: Arc<Document>, _capability: &str) {
    tracing::trace!("TODO: implement WebSocket `nodes` protocol connection");
}

/// Handle a WebSocket connection using the "object" protocol
#[tracing::instrument(skip(ws, doc))]
async fn websocket_object_protocol(ws: WebSocket, doc: Arc<Document>, capability: &str) {
    tracing::trace!("WebSocket `object` protocol connection");

    let (ws_sender, ws_receiver) = ws.split();

    let (in_sender, in_receiver) = channel(8);
    receive_websocket_messages(ws_receiver, in_sender);

    let (out_sender, out_receiver) = channel(1024);
    send_websocket_messages(out_receiver, ws_sender);

    if let Err(error) = doc.sync_object(in_receiver, out_sender).await {
        tracing::error!("While syncing object for WebSocket client: {error}")
    }
}

/// Handle a WebSocket connection using the "directory" protocol
#[tracing::instrument(skip(ws, doc))]
async fn websocket_directory_protocol(
    ws: WebSocket,
    doc: Arc<Document>,
    capability: &str,
    dir: PathBuf,
) {
    tracing::trace!("WebSocket `directory` protocol connection");

    let (ws_sender, ws_receiver) = ws.split();

    let (in_sender, in_receiver) = channel(8);
    receive_websocket_messages(ws_receiver, in_sender);

    let (out_sender, out_receiver) = channel(8);
    send_websocket_messages(out_receiver, ws_sender);

    if let Err(error) = doc.sync_directory(dir, in_receiver, out_sender).await {
        tracing::error!("While syncing directory for WebSocket client: {error}")
    }
}

/// Handle a WebSocket connection using the "dom" protocol
#[tracing::instrument(skip(ws, doc))]
async fn websocket_dom_protocol(ws: WebSocket, doc: Arc<Document>) {
    tracing::trace!("WebSocket `dom` protocol connection");

    let (ws_sender, ws_receiver) = ws.split();

    let (in_sender, in_receiver) = channel(1024);
    receive_websocket_messages(ws_receiver, in_sender);

    let (out_sender, out_receiver) = channel(1024);
    send_websocket_messages(out_receiver, ws_sender);

    if let Err(error) = doc.sync_dom(None, in_receiver, out_sender).await {
        tracing::error!("While syncing DOM for WebSocket client: {error}")
    }
}

/// Handle a WebSocket connection using a "format" protocol
#[tracing::instrument(skip(ws, doc))]
async fn websocket_format_protocol(
    ws: WebSocket,
    doc: Arc<Document>,
    capability: &str,
    format: &str,
) {
    tracing::trace!("WebSocket `format` protocol connection");

    let (ws_sender, ws_receiver) = ws.split();

    let (in_sender, in_receiver) = channel(1024);
    receive_websocket_messages(ws_receiver, in_sender);

    let (out_sender, out_receiver) = channel(1024);
    send_websocket_messages(out_receiver, ws_sender);

    let format = format.parse().ok();

    let decode_options = DecodeOptions {
        format: format.clone(),
        ..Default::default()
    };

    let encode_options = EncodeOptions {
        format,
        ..Default::default()
    };

    if let Err(error) = doc
        .sync_format(
            Some(in_receiver),
            Some(out_sender),
            Some(decode_options),
            Some(encode_options),
        )
        .await
    {
        tracing::error!("While syncing string for WebSocket client: {error}")
    }
}

/// Receive WebSocket messages and forward to a channel
#[tracing::instrument(skip_all)]
fn receive_websocket_messages<T>(mut receiver: SplitStream<WebSocket>, sender: Sender<T>)
where
    T: DeserializeOwned + Send + 'static,
{
    tracing::trace!("Receiving WebSocket messages");

    tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            tracing::trace!("Received WebSocket message");

            let message = match message {
                Message::Text(message) => message,
                Message::Close(..) => {
                    tracing::debug!("WebSocket connection closed");
                    break;
                }
                _ => continue,
            };

            let message = match serde_json::from_str(&message) {
                Ok(message) => message,
                Err(error) => {
                    tracing::error!(
                        "Unable to deserialize `{}` message: {error}",
                        std::any::type_name::<T>()
                    );
                    continue;
                }
            };

            if sender.send(message).await.is_err() {
                break;
            }
        }
    });
}

/// Send WebSocket messages forwarded from a channel
#[tracing::instrument(skip_all)]
fn send_websocket_messages<T>(mut receiver: Receiver<T>, mut sender: SplitSink<WebSocket, Message>)
where
    T: Serialize + Send + 'static,
{
    tracing::trace!("Sending WebSocket messages");

    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            tracing::trace!("Sending WebSocket message");

            let message = match serde_json::to_string(&message) {
                Ok(message) => message,
                Err(error) => {
                    tracing::error!(
                        "Unable to serialize `{}` message: {error}",
                        std::any::type_name::<T>()
                    );
                    continue;
                }
            };

            let message = Message::Text(message.into());

            if sender.send(message).await.is_err() {
                break;
            }
        }
    });
}

// Create a 404 Not Found response
fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Not found").into_response()
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use eyre::Result;

    use super::*;

    /// Test the `resolve_path` method using the `routing` example
    /// Skip on windows because of path incompatibility (\ v /)
    #[cfg(not(target_os = "windows"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_resolve_path() -> Result<()> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/projects/routing")
            .canonicalize()?;

        // Will forbid paths with `..` in them
        for path in [
            "..",
            "../..",
            "some/..",
            "../some",
            "some/../some",
            "some/../..",
        ] {
            let response = serve_path(
                State(ServerState::default()),
                Path(path.to_string()),
                Default::default(),
            )
            .await?;
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }

        // Will return 404 for private files and any files in private folders,
        // even with `raw` true
        for path in [
            "_private",
            "_private.md",
            "_private/README",
            "_private/README.md",
            "birds/jay/_private",
            "birds/jay/_private.md",
        ] {
            let response = serve_path(
                State(ServerState {
                    dir: dir.clone(),
                    raw: true,
                    ..Default::default()
                }),
                Path(path.to_string()),
                Default::default(),
            )
            .await?;
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Resolved `{path}` but should not have"
            );
        }

        // Will serve files when `raw` flag is `true`, but will 403 otherwise
        let query = Query(HashMap::from([("mode".to_string(), "raw".to_string())]));
        for (path, mime) in [
            ("README.md", "text/markdown"),
            ("bird/jay/index.json5", "application/json5"),
            ("bird/owl/README.md", "text/markdown"),
        ] {
            let response = serve_path(
                State(ServerState {
                    dir: dir.clone(),
                    raw: true,
                    ..Default::default()
                }),
                Path(path.to_string()),
                query.clone(),
            )
            .await?;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                response.headers().get("content-type"),
                Some(&HeaderValue::from_static(mime))
            );

            let response = serve_path(
                State(ServerState {
                    dir: dir.clone(),
                    raw: false,
                    ..Default::default()
                }),
                Path(path.to_string()),
                query.clone(),
            )
            .await?;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }

        // Will route a path to a file with a matching stem according to rules
        // regarding format precedence and modification times
        for (path, source) in [
            ("bird", "bird/index.smd"),
            ("bird/kea", "bird/kea.smd"),
            ("bird/jay", "bird/jay/index.json5"),
            ("bird/owl", "bird/owl/README.md"),
        ] {
            let response = serve_path(
                State(ServerState {
                    dir: dir.clone(),
                    source: true,
                    ..Default::default()
                }),
                Path(path.to_string()),
                Default::default(),
            )
            .await?;
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Did not resolve `{path}`"
            );
            assert_eq!(
                response.headers().get("sourcemap"),
                Some(&HeaderValue::from_static(source))
            );
        }

        Ok(())
    }
}
