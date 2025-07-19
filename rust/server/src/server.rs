use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, Request, StatusCode},
    middleware::{from_fn_with_state as middleware_fn, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use rand::{rng, Rng};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::trace::TraceLayer;

use common::{
    clap::{self, Args},
    eyre::{self},
    serde::Deserialize,
    smart_default::SmartDefault,
    tokio::{net::TcpListener, sync::mpsc},
    tracing,
};
use document::SyncDirection;
pub(crate) use version::STENCILA_VERSION;

use crate::{
    auth,
    documents::{self, Documents},
    login, statics,
};

/// Server state available from all routes
#[derive(Default, Clone)]
pub(crate) struct ServerState {
    /// The directory that is being served
    pub dir: PathBuf,

    /// The `server_token` for the server
    pub server_token: Option<String>,

    /// Whether files should be served raw
    pub raw: bool,

    /// Whether the `SourceMap` header should be set for document responses
    pub source: bool,

    /// Whether and in which direction(s) to sync served documents with
    /// the file system
    pub sync: Option<SyncDirection>,

    /// The cache of documents
    pub docs: Arc<Documents>,

    /// Shutdown signal sender for graceful server termination
    pub shutdown_sender: Option<mpsc::Sender<()>>,
}

/// Run the HTTP/Websocket server
#[derive(Debug, SmartDefault, Args)]
pub struct ServeOptions {
    /// The directory to serve
    ///
    /// Defaults to the current working directory
    #[arg(default_value = ".")]
    #[default(".")]
    pub dir: PathBuf,

    /// The address to serve on
    ///
    /// Defaults to `127.0.0.1` (localhost), use `0.0.0.0` to listen
    /// on all addresses.
    #[arg(long, short, default_value = "127.0.0.1")]
    #[default(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    pub address: IpAddr,

    /// The port to serve on
    ///
    /// Defaults to port 9000.
    #[arg(long, short, default_value_t = 9000)]
    #[default(9000)]
    pub port: u16,

    /// Do not authenticate or authorize requests
    ///
    /// By default, requests to all routes (except `~static/*`) require
    /// an access token.
    #[arg(long)]
    pub no_auth: bool,

    /// Should files be served raw?
    ///
    /// When `true` and a request is made to a path that exists within `dir`,
    /// the file will be served with a `Content-Type` header corresponding to
    /// the file's extension.
    #[arg(long)]
    pub raw: bool,

    /// Should `SourceMap` headers be sent?
    ///
    /// When `true`, then the `SourceMap` header will be set with the URL
    /// of the document that was rendered as HTML. Usually only useful if
    /// `raw` is also `true`.
    #[arg(long)]
    pub source: bool,

    /// Whether and in which direction(s) to sync served documents
    #[arg(long)]
    pub sync: Option<SyncDirection>,

    /// The server token to use
    ///
    /// This is not a CLI argument. It is only passed to the `serve()` function
    /// when it is called internally.
    #[clap(skip)]
    pub server_token: Option<String>,

    /// Do not show a startup message giving a login URL
    pub no_startup_message: bool,

    /// Whether the server can be be gracefully shutdown by sending
    /// a message on the server state's `shutdown_sender`.
    pub graceful_shutdown: bool,
}

/// Start the server
pub async fn serve(
    ServeOptions {
        address,
        port,
        dir,
        no_auth,
        raw,
        source,
        sync,
        server_token,
        no_startup_message,
        graceful_shutdown,
    }: ServeOptions,
) -> eyre::Result<()> {
    let dir = dir.canonicalize()?;

    let address = SocketAddr::new(address, port);
    let server_token = if no_auth {
        tracing::warn!("Using `--no-auth` flag; no routes are protected by authentication/authorization checks");
        None
    } else {
        Some(server_token.unwrap_or_else(get_server_token))
    };

    let mut url = format!("http://{address}");
    if let Some(sst) = &server_token {
        url.push_str("/~login?sst=");
        url.push_str(sst);
    }

    let (shutdown_sender, shutdown_receiver) = if graceful_shutdown {
        let channel = mpsc::channel(10);
        (Some(channel.0), Some(channel.1))
    } else {
        (None, None)
    };

    let state = ServerState {
        dir,
        server_token,
        raw,
        source,
        sync,
        shutdown_sender,
        ..Default::default()
    };

    let router = Router::new()
        .nest("/~static", statics::router())
        .route("/~login", get(login::login))
        .nest("/~auth", auth::router())
        .nest(
            "/~documents",
            documents::router().route_layer(middleware_fn(state.clone(), auth_middleware)),
        )
        .route(
            "/{*path}",
            get(documents::serve_path).route_layer(middleware_fn(state.clone(), auth_middleware)),
        )
        .route(
            "/",
            get(documents::serve_root).route_layer(middleware_fn(state.clone(), auth_middleware)),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .with_state(state)
        .into_make_service();

    let listener = TcpListener::bind(&address).await?;

    if !no_startup_message {
        tracing::info!("Starting server at {url}");
    }

    if let Some(mut shutdown_receiver) = shutdown_receiver {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                if let Some(()) = shutdown_receiver.recv().await {
                    tracing::debug!("Server shutdown signal received, stopping server gracefully");
                } else {
                    tracing::warn!("Server shutdown channel closed without signal");
                }
            })
            .await?;
    } else {
        axum::serve(listener, router).await?;
    }

    Ok(())
}

/// Get or generate a server token token
///
/// If the `STENCILA_SERVER_TOKEN` environment variable is present
/// will use that, otherwise will generate a random access token.
pub fn get_server_token() -> String {
    if let Ok(token) = env::var("STENCILA_SERVER_TOKEN") {
        return token;
    }

    // Avoid non-word characters for easier copy/paste
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                           abcdefghijklmnopqrstuvwxyz\
                           0123456789";

    // Long enough to make brute force attacks very, very slow while not being
    // too unwieldy to use in URLs
    const LEN: usize = 60;

    let mut rng = rng();
    let random: String = (0..LEN)
        .map(|_| {
            let idx = rng.random_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect();

    // Prefix with `sst_` (Stencila server token) to avoid confusion with
    // `sat_` (Stencila access token) generated by Stencila Cloud.
    ["sst_", &random].concat()
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct AuthQuery {
    sst: Option<String>,
}

/// Authentication / authorization middleware function
///
/// Currently only does authorization, based on an Stencila server token (sst).
/// In the future, may do authentication (using JWTs for example).
async fn auth_middleware(
    State(state): State<ServerState>,
    cookies: Cookies,
    Query(query): Query<AuthQuery>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let Some(server_token) = state.server_token else {
        return Ok(next.run(request).await);
    };

    // Check if the access token is provided as an Authorization header
    if let Some(auth_header) = headers.get("Authorization") {
        if auth_header.to_str().unwrap_or_default() == ["Token ", &server_token].concat() {
            return Ok(next.run(request).await);
        }
    }

    // Check if the access token is provided as a cookie
    if let Some(cookie) = cookies.get("sst") {
        if cookie.value() == server_token {
            return Ok(next.run(request).await);
        }
    }

    // Check if the access token is provided as a query parameter
    if let Some(token) = query.sst {
        if token == *server_token {
            // Set the access token as a cookie. Setting path is
            // important so that the cookie is sent for all routes
            // including document websocket connections
            let mut cookie = Cookie::new("sst", token);
            cookie.set_path("/");
            cookies.add(cookie);

            return Ok(next.run(request).await);
        }
    }

    Err((StatusCode::UNAUTHORIZED, "Unauthorized").into_response())
}
