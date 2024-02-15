use std::{
    cmp::Ordering,
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    path::{Component, PathBuf},
    str::FromStr,
    sync::Arc,
    time::UNIX_EPOCH,
};

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::{
        header::{HeaderName, ACCEPT_ENCODING, CACHE_CONTROL, CONTENT_ENCODING, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use document::{Document, DocumentId, SyncDirection};
use rust_embed::RustEmbed;

use codecs::{DecodeOptions, EncodeOptions};
use common::{
    clap::{self, Args},
    eyre::{self, OptionExt},
    futures::{
        stream::{SplitSink, SplitStream},
        SinkExt, StreamExt,
    },
    glob::{glob, glob_with, MatchOptions},
    itertools::Itertools,
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
    tokio::{
        self,
        fs::read,
        net::TcpListener,
        sync::mpsc::{channel, Receiver, Sender},
    },
    tracing,
};
use format::Format;
use tower_http::trace::TraceLayer;

use crate::documents::Documents;
use crate::errors::InternalError;

/// The current version of Stencila
///
/// Used to improving browser caching of assets by
/// serving static files using versioned paths.
const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The encodings to use when serving static files
///
/// In development do not serve Brotli or Gzip because `make -C web watch` does not
/// build those compressed files (only `make -C web build` does).
#[cfg(debug_assertions)]
const STATIC_ENCODINGS: [(&str, &str); 1] = [("", "")];
#[cfg(not(debug_assertions))]
const STATIC_ENCODINGS: [(&str, &str); 3] = [("br", ".br"), ("gzip", ".gz"), ("", "")];

/// Embedded static files
///
/// During development these are served directly from the folder
/// but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../web/dist"]
#[cfg_attr(not(debug_assertions), exclude = "*.map")]
#[exclude = ".gitignore"]
struct Static;

/// Server state available from all routes
#[derive(Default, Clone)]
struct ServerState {
    /// The directory that is being served
    dir: PathBuf,

    /// Whether files should be served raw
    raw: bool,

    /// Whether the `SourceMap` header should be set for document responses
    source: bool,

    /// Whether and in which direction(s) to sync served documents with
    /// the file system
    sync: Option<SyncDirection>,

    /// The cache of documents
    docs: Arc<Documents>,
}

/// Options for the `serve` function
#[derive(Debug, Args)]
pub struct ServeOptions {
    /// The directory to serve
    ///
    /// Defaults to the current working directory
    #[arg(default_value = ".")]
    dir: PathBuf,

    /// The address to serve on
    ///
    /// Defaults to `127.0.0.1` (localhost), use `0.0.0.0` to listen
    /// on all addresses.
    #[arg(long, short, default_value = "127.0.0.1")]
    address: IpAddr,

    /// The port to serve on
    ///
    /// Defaults to port 9000.
    #[arg(long, short, default_value_t = 9000)]
    port: u16,

    /// Should files be served raw?
    ///
    /// When `true` and a request is made to a path that exists within `dir`,
    /// the file will be served with a `Content-Type` header corresponding to
    /// the file's extension.
    #[arg(long)]
    raw: bool,

    /// Should `SourceMap` headers be sent?
    ///
    /// When `true`, then the `SourceMap` header will be set with the URL
    /// of the document that was rendered as HTML. Usually only useful if
    /// `raw` is also `true`.
    #[arg(long)]
    source: bool,

    /// Whether and in which direction(s) to sync served documents
    #[arg(long)]
    sync: Option<SyncDirection>,
}

/// Start the server
pub async fn serve(
    ServeOptions {
        address,
        port,
        dir,
        raw,
        source,
        sync,
    }: ServeOptions,
) -> eyre::Result<()> {
    let address = SocketAddr::new(address, port);
    let dir = dir.canonicalize()?;

    let router = Router::new()
        .route("/~static/*path", get(serve_static))
        .route("/~open/*path", get(serve_open))
        .route("/~close/*id", post(serve_close))
        .route("/~export/*id", get(serve_export))
        .route("/~ws/*id", get(serve_ws))
        .route("/*path", get(serve_document))
        .route("/", get(serve_home))
        .layer(TraceLayer::new_for_http())
        .with_state(ServerState {
            dir,
            raw,
            source,
            sync,
            ..Default::default()
        });

    let listener = TcpListener::bind(&address).await?;

    tracing::info!("Starting server at http://{address}");
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}

/// Get a static file (e.g. `index.js``)
///
/// Paths to static files include a version so that, in production, the cache control
/// header can be set such that clients should only ever need to make a single request
/// for each version of a static file.
///
/// This cache control is turned off in development so that changes to those files
/// propagate to the browser.
#[tracing::instrument]
async fn home() -> Response {
    let static_version = if cfg!(debug_assertions) {
        "dev"
    } else {
        STENCILA_VERSION
    };

    let page = format!(
        r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <title>Stencila</title>
    <link rel="stylesheet" href="/static/{static_version}/index.css" />
    <script type="module" src="/static/{static_version}/index.js"></script>
</head>
<body>
</body>
</html>"#
    );

    Html(page).into_response()
}

/// Serve a static file (e.g. `index.js``)
///
/// Paths to static files include a version so that, in production, the cache control
/// header can be set such that clients should only ever need to make a single request
/// for each version of a static file.
///
/// This cache control is turned off in development so that changes to those files
/// propagate to the browser.
#[tracing::instrument]
async fn serve_static(
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<Response, InternalError> {
    let path = path.split_once('/').map(|(version, rest)| {
        if version != "dev" && version != STENCILA_VERSION {
            tracing::warn!("Request was made for a different version (current {STENCILA_VERSION}) of a static file: {path}")
        }
        rest.to_string()
    }).unwrap_or(path);

    let accept_encoding = headers
        .get(ACCEPT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    for (encoding, ext) in STATIC_ENCODINGS {
        if accept_encoding.contains(encoding) {
            let asset_path = [&path, ext].concat();
            if let Some(file) = Static::get(&asset_path) {
                let content_type = mime_guess::from_path(path).first_or_octet_stream();

                let mut response =
                    Response::builder().header(CONTENT_TYPE, content_type.essence_str());

                if !encoding.is_empty() {
                    response = response.header(CONTENT_ENCODING, encoding);
                }

                if !cfg!(debug_assertions) {
                    response = response.header(CACHE_CONTROL, "max-age=31536000, immutable");
                }

                return response
                    .body(Body::from(file.data))
                    .map_err(InternalError::new);
            }
        }
    }

    Ok(StatusCode::NOT_FOUND.into_response())
}

/// Resolve a URL path into a file or directory path
///
/// This is an interim implementation and is likely to be replaced with
/// an implementation which uses a tries and which handles parameterized routes.
fn resolve_path(path: PathBuf) -> Result<Option<PathBuf>, InternalError> {
    // If the path ends with `*` and a directory exists there then resolve to it
    if let Some(path) = path.to_string_lossy().strip_suffix('*') {
        let path = PathBuf::from(path);
        if path.exists() && path.is_dir() {
            return Ok(Some(path));
        }
    }

    // If a file exists at the path then just resolve to it
    if path.exists() && path.is_file() {
        return Ok(Some(path));
    }

    // If any files have the same stem as the path (everything minus the extension)
    // then use the one with the format with highest precedence and latest modification date.
    // This checks that the file has a stem otherwise files like `.gitignore` match against it.
    let pattern = format!("{}.*", path.display());
    if let Some(path) = glob(&pattern)
        .map_err(InternalError::new)?
        .flatten()
        .filter(|path| {
            path.file_name()
                .map_or(false, |name| !name.to_string_lossy().starts_with('.'))
                && path.is_file()
        })
        .sorted_by(|a, b| {
            let a_format = Format::from_path(a).unwrap_or_default();
            let b_format = Format::from_path(b).unwrap_or_default();
            match a_format.rank().cmp(&b_format.rank()) {
                Ordering::Equal => {
                    let a_modified = std::fs::metadata(a)
                        .and_then(|metadata| metadata.modified())
                        .unwrap_or(UNIX_EPOCH);
                    let b_modified = std::fs::metadata(b)
                        .and_then(|metadata| metadata.modified())
                        .unwrap_or(UNIX_EPOCH);
                    a_modified.cmp(&b_modified).reverse()
                }
                ordering => ordering,
            }
        })
        .next()
    {
        return Ok(Some(path));
    }

    // If the path correlates to a folder with an index, main, or readme file
    // then use the one with the highest precedence
    let pattern = format!("{}/*", path.display());
    if let Some(path) = glob_with(
        &pattern,
        MatchOptions {
            case_sensitive: false,
            ..Default::default()
        },
    )
    .map_err(InternalError::new)?
    .flatten()
    .find(|path| {
        // Select the first file matching these criteria
        // noting that `glob` returns entries sorted alphabetically
        path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| {
                    let name = name.to_lowercase();
                    name.starts_with("index.")
                        || name.starts_with("main.")
                        || name.starts_with("readme.")
                })
                .unwrap_or_default()
    }) {
        return Ok(Some(path));
    }

    Ok(None)
}

/// Open a document and return its id
#[tracing::instrument(skip(docs))]
async fn serve_open(
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
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    // Resolve the URL path into a filesystem path
    let path = resolve_path(path)?;

    // Return early if no path resolved
    let Some(path) = path else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // Get the document for the path
    let doc = docs
        .by_path(&path, sync)
        .await
        .map_err(InternalError::new)?;
    let doc_id = doc.id();

    #[derive(Serialize)]
    #[serde(crate = "common::serde")]
    struct OpenResponse {
        id: String,
    }

    Ok(Json(OpenResponse {
        id: doc_id.to_string(),
    })
    .into_response())
}

/// Serve a document
#[tracing::instrument(skip(docs))]
async fn serve_document(
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
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    // Resolve the URL path into a filesystem path
    let path = resolve_path(path)?;

    // Return early if no path resolved
    let Some(path) = path else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // Get the document for the path
    let doc = docs
        .by_path(&path, sync)
        .await
        .map_err(InternalError::new)?;
    let doc_id = doc.id();
    let name = path
        .file_name()
        .ok_or_eyre("File has no name")
        .map_err(InternalError::new)?
        .to_string_lossy();

    // Get various query parameters
    let mode = query
        .get("mode")
        .map_or("app", |value: &String| value.as_ref());
    let view = query
        .get("view")
        .map_or("static", |value: &String| value.as_ref());
    // TODO: restrict the access to the highest based on the user's role
    let access = query.get("access").map_or("write", |value| value.as_ref());
    let theme = query
        .get("theme")
        .map_or("default", |value: &String| value.as_ref());
    let format = query
        .get("format")
        .map_or("markdown", |value| value.as_ref());

    // Generate the body of the HTML (or an early-returned response for `raw` view)
    let body = if mode == "raw" {
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
    } else if mode == "doc" {
        if let "static" | "print" = view {
            doc.export(
                None,
                Some(EncodeOptions {
                    format: Some(Format::Dom),
                    ..Default::default()
                }),
            )
            .await
            .map_err(InternalError::new)?
        } else {
            format!("<stencila-{view}-view doc={doc_id} view={view} access={access} theme={theme} format={format}></stencila-{view}-view>")
        }
    } else {
        format!(
            r#"<stencila-main-app docs='[{{"docId":"{doc_id}","path":"{path}","name":"{name}"}}]' view={view} access={access} theme={theme} format={format}></stencila-main-app>"#,
            path = path.display()
        )
    };

    // The version path segment for static assets (JS & CSS)
    let version = if cfg!(debug_assertions) {
        "dev"
    } else {
        STENCILA_VERSION
    };

    // The stylesheet tag for the theme
    // TODO: resolve the theme for the document
    let theme_tag = format!(
        r#"<link title="theme:{theme}" rel="stylesheet" type="text/css" href="/~static/{version}/themes/{theme}.css">"#
    );

    // The script tag for the view or app
    let extra_head = if mode == "doc" {
        if view == "static" {
            // No need for any JS in this mode for this view
            String::new()
        } else if view == "print" {
            format!(
                r#"<link rel="stylesheet" type="text/css" href="/~static/{version}/views/print.css">
                   <script type="module" src="/~static/{version}/views/print.js"></script>"#
            )
        } else {
            format!(r#"<script type="module" src="/~static/{version}/views/{view}.js"></script>"#)
        }
    } else if mode == "app" {
        format!(
            r#" <link rel="preconnect" href="https://fonts.googleapis.com">
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
                <link href="https://fonts.googleapis.com/css2?family=Lato:wght@400;900&family=Montserrat:wght@400;600&display=swap" rel="stylesheet">
                <link rel="stylesheet" type="text/css" href="/~static/{version}/shoelace-style/themes/light.css">
                <link rel="stylesheet" type="text/css" href="/~static/{version}/shoelace-style/themes/dark.css">
                <link rel="stylesheet" type="text/css" href="/~static/{version}/apps/main.css">
                <script type="module" src="/~static/{version}/apps/main.js"></script>"#
        )
    } else {
        String::new()
    };

    let html = format!(
        r#"<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8"/>
        <title>Stencila</title>
        <link rel="icon" type="image/png" href="/~static/{version}/images/favicon.png">
        {theme_tag}
        {extra_head}
    </head>
    <body>
        {body}
    </body>
</html>"#
    );

    // Build the response
    let response = (|| -> eyre::Result<Response> {
        let mut response = Response::builder()
        // TODO set the content type header
        //.header(CONTENT_TYPE, format.media_type())
        ;

        if source {
            if let Ok(path) = path.strip_prefix(&dir) {
                response = response.header(
                    HeaderName::try_from("SourceMap")?,
                    HeaderValue::from_str(&path.to_string_lossy())?,
                );
            }
        }

        Ok(response.body(Body::from(html))?)
    })()
    .map_err(InternalError::new)?;

    Ok(response)
}

/// Serve the home
///
/// Only exists to serve the "main" (if any) for the home
/// directory of the server e.g. a README in a repo.
#[tracing::instrument(skip_all)]
async fn serve_home(
    state: State<ServerState>,
    query: Query<HashMap<String, String>>,
) -> Result<Response, InternalError> {
    serve_document(state, Path(String::new()), query).await
}

/// Handle a request to close a document
async fn serve_close(
    State(ServerState { docs, .. }): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Response, InternalError> {
    let Ok(id) = DocumentId::from_str(&id) else {
        return Ok((StatusCode::BAD_REQUEST, "Invalid document id").into_response());
    };

    docs.close(&id).await.map_err(InternalError::new)?;

    Ok(StatusCode::OK.into_response())
}

/// Handle a request to export a document
///
/// TODO: This should add correct MIME type to response
/// and handle binary formats.
async fn serve_export(
    State(ServerState { docs, .. }): State<ServerState>,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, InternalError> {
    let Ok(id) = DocumentId::from_str(&id) else {
        return Ok((StatusCode::BAD_REQUEST, "Invalid document id").into_response());
    };

    let doc = docs.by_id(&id).await.map_err(InternalError::new)?;

    let format = query
        .get("format")
        .and_then(|format| Format::from_name(format).ok());

    let dom = query.get("dom").and_then(|dom| dom.parse().ok());

    let options = EncodeOptions {
        format,
        dom,
        ..Default::default()
    };

    let content = doc
        .export(None, Some(options))
        .await
        .map_err(InternalError::new)?;

    Ok(content.into_response())
}

/// Handle a WebSocket upgrade request
async fn serve_ws(
    State(ServerState {
        dir, docs, sync, ..
    }): State<ServerState>,
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
) -> Result<Response, InternalError> {
    let Ok(id) = DocumentId::from_str(&id) else {
        return Ok((StatusCode::BAD_REQUEST, "Invalid document id").into_response());
    };

    let doc = docs.by_id(&id).await.map_err(InternalError::new)?;

    // TODO: Change the allowed protocols based on the users permissions
    let mut protocols = vec![
        "read.dom.stencila.org".to_string(),
        "read.debug.stencila.org".to_string(),
        "read.object.stencila.org".to_string(),
    ];

    // Protocols only permitted if sync direction includes `Out`
    if matches!(sync, Some(SyncDirection::Out | SyncDirection::InOut)) {
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
        Format::Yaml,
    ] {
        protocols.push(format!("read.{format}.stencila.org"));
        protocols.push(format!("write.{format}.stencila.org"));
        protocols.push(format!("write.{format}.stencila.org"));
    }

    for access in [
        "comment", "suggest", "input", "code", "prose", "write", "admin",
    ] {
        protocols.push(format!("{access}.nodes.stencila.org"));
    }

    let response = ws
        .protocols(protocols)
        .on_upgrade(move |ws| handle_ws(ws, doc, dir));

    Ok(response)
}

/// Handle a WebSocket connection
#[tracing::instrument(skip(ws, doc))]
async fn handle_ws(ws: WebSocket, doc: Arc<Document>, dir: PathBuf) {
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
        handle_ws_nodes(ws, doc, capability).await;
    } else if format == "object" {
        handle_ws_object(ws, doc, capability).await;
    } else if format == "directory" {
        handle_ws_directory(ws, doc, dir).await;
    } else {
        handle_ws_format(ws, doc, capability, format).await;
    }
}

/// Handle a WebSocket connection using the "nodes" protocol
#[tracing::instrument(skip(ws, doc))]
async fn handle_ws_nodes(ws: WebSocket, doc: Arc<Document>, capability: &str) {
    tracing::trace!("WebSocket `nodes` connection");

    let (.., ws_receiver) = ws.split();

    let (in_sender, in_receiver) = channel(1024);
    receive_ws_messages(ws_receiver, in_sender);

    if let Err(error) = doc.sync_nodes(in_receiver).await {
        tracing::error!("While syncing nodes for WebSocket client: {error}")
    }
}

/// Handle a WebSocket connection using the "object" protocol
#[tracing::instrument(skip(ws, doc))]
async fn handle_ws_object(ws: WebSocket, doc: Arc<Document>, capability: &str) {
    tracing::trace!("WebSocket `object` connection");

    let (ws_sender, ws_receiver) = ws.split();

    let (in_sender, in_receiver) = channel(8);
    receive_ws_messages(ws_receiver, in_sender);

    let (out_sender, out_receiver) = channel(1024);
    send_ws_messages(out_receiver, ws_sender);

    if let Err(error) = doc.sync_object(in_receiver, out_sender).await {
        tracing::error!("While syncing object for WebSocket client: {error}")
    }
}

/// Handle a WebSocket connection using the "directory" protocol
#[tracing::instrument(skip(ws, doc))]
async fn handle_ws_directory(ws: WebSocket, doc: Arc<Document>, dir: PathBuf) {
    tracing::trace!("WebSocket `directory` connection");

    let (ws_sender, ws_receiver) = ws.split();

    let (in_sender, in_receiver) = channel(8);
    receive_ws_messages(ws_receiver, in_sender);

    let (out_sender, out_receiver) = channel(8);
    send_ws_messages(out_receiver, ws_sender);

    if let Err(error) = doc.sync_directory(dir, in_receiver, out_sender).await {
        tracing::error!("While syncing directory for WebSocket client: {error}")
    }
}

/// Handle a WebSocket connection using a "format" protocol
#[tracing::instrument(skip(ws, doc))]
async fn handle_ws_format(ws: WebSocket, doc: Arc<Document>, capability: &str, format: &str) {
    tracing::trace!("WebSocket `format` connection");

    let (ws_sender, ws_receiver) = ws.split();

    let (in_sender, in_receiver) = channel(1024);
    receive_ws_messages(ws_receiver, in_sender);

    let (out_sender, out_receiver) = channel(1024);
    send_ws_messages(out_receiver, ws_sender);

    let format = format.parse().ok();

    let decode_options = DecodeOptions {
        format,
        ..Default::default()
    };

    let encode_options = EncodeOptions {
        format,
        compact: Some(false),
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
fn receive_ws_messages<T>(mut receiver: SplitStream<WebSocket>, sender: Sender<T>)
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
fn send_ws_messages<T>(mut receiver: Receiver<T>, mut sender: SplitSink<WebSocket, Message>)
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

            let message = Message::Text(message);

            if sender.send(message).await.is_err() {
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use common::{eyre::Result, tokio};

    use super::*;

    /// Test the `resolve_path` method using the `routing` example
    #[tokio::test]
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
            let response = serve_document(
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
            let response = serve_document(
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
            let response = serve_document(
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

            let response = serve_document(
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
            ("bird", "bird/index.md"),
            ("bird/kea", "bird/kea.md"),
            ("bird/jay", "bird/jay/index.json5"),
            ("bird/owl", "bird/owl/README.md"),
        ] {
            let response = serve_document(
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
