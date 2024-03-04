use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

use common::{
    clap::{self, Args},
    eyre::{self},
    tokio::net::TcpListener,
    tracing,
};
use document::SyncDirection;

use crate::{
    documents::{self, Documents},
    secrets, statics,
};

/// The current version of Stencila
///
/// Used to improving browser caching of assets by
/// serving static files using versioned paths.
pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Server state available from all routes
#[derive(Default, Clone)]
pub(crate) struct ServerState {
    /// The directory that is being served
    pub dir: PathBuf,

    /// Whether files should be served raw
    pub raw: bool,

    /// Whether the `SourceMap` header should be set for document responses
    pub source: bool,

    /// Whether and in which direction(s) to sync served documents with
    /// the file system
    pub sync: Option<SyncDirection>,

    /// The cache of documents
    pub docs: Arc<Documents>,
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
        .nest("/~static", statics::router())
        .nest("/~documents", documents::router())
        .nest("/~secrets", secrets::router())
        .route("/*path", get(documents::serve_path))
        .route("/", get(documents::serve_root))
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
