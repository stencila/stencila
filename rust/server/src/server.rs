use std::{
    env, fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    process,
    sync::Arc,
    time::SystemTime,
};

use axum::{
    Router,
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, Method, Request, StatusCode, header::HeaderValue},
    middleware::{Next, from_fn_with_state as middleware_fn},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Args;
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use tokio::{net::TcpListener, sync::mpsc};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use stencila_dirs::{DirType, get_app_dir};
use stencila_document::SyncDirection;
pub(crate) use stencila_version::STENCILA_VERSION;

use crate::{
    auth,
    documents::{self, Documents},
    login, statics, themes,
};

/// Server runtime information written to disk for discovery
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Process ID of the server
    pub pid: u32,

    /// Port the server is listening on
    pub port: u16,

    /// Server authentication token
    pub token: Option<String>,

    /// Directory being served (absolute path)
    pub directory: PathBuf,

    /// Unix timestamp when server started
    pub started_at: u64,
}

impl ServerInfo {
    /// Create ServerInfo for current server
    fn new(port: u16, token: Option<String>, directory: PathBuf) -> Self {
        Self {
            pid: process::id(),
            port,
            token,
            directory,
            started_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Write server info to cache directory
    fn write(&self) -> eyre::Result<PathBuf> {
        let servers_dir = get_app_dir(DirType::Servers, true)?;
        let info_path = servers_dir.join(format!("{}.json", self.pid));

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&info_path, json)?;

        // Set permissions to 600 (owner read/write only) for security
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&info_path, permissions)?;
        }

        tracing::debug!("Wrote server info to {}", info_path.display());

        Ok(info_path)
    }

    /// Remove server info file
    fn cleanup(&self) -> eyre::Result<()> {
        if let Ok(servers_dir) = get_app_dir(DirType::Servers, false) {
            let info_path = servers_dir.join(format!("{}.json", self.pid));

            if info_path.exists() {
                fs::remove_file(&info_path)?;
                tracing::debug!("Cleaned up server info at {}", info_path.display());
            }
        }
        Ok(())
    }
}

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

/// CORS policy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, SmartDefault)]
pub enum CorsLevel {
    /// No CORS headers
    #[default]
    None,
    /// Allow only same-origin requests
    Restrictive,
    /// Allow localhost and 127.0.0.1 origins only
    Local,
    /// Allow all origins, methods, and headers
    Permissive,
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

    /// CORS policy level
    ///
    /// Controls Cross-Origin Resource Sharing (CORS) headers.
    /// Ordered from most to least restrictive:
    /// - `none`: No CORS headers (default)
    /// - `restrictive`: Allow GET and POST requests from localhost
    /// - `local`: Allow any requests from localhost and 127.0.0.1 origins
    /// - `permissive`: Allow all origins, methods, and headers
    #[arg(long, default_value = "none")]
    #[default(CorsLevel::None)]
    pub cors: CorsLevel,

    /// The server token to use
    ///
    /// This is not a CLI argument. It is only passed to the `serve()` function
    /// when it is called internally.
    #[clap(skip)]
    pub server_token: Option<String>,

    /// Do not show a startup message giving a login URL
    #[clap(skip)]
    pub no_startup_message: bool,
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
        cors,
        server_token,
        no_startup_message,
    }: ServeOptions,
) -> eyre::Result<()> {
    let dir = dir.canonicalize()?;

    let address = SocketAddr::new(address, port);
    let server_token = if no_auth {
        tracing::warn!(
            "Using `--no-auth` flag; no routes are protected by authentication/authorization checks"
        );
        None
    } else {
        Some(server_token.unwrap_or_else(get_server_token))
    };

    let mut url = format!("http://{address}");
    if let Some(sst) = &server_token {
        url.push_str("/~login?sst=");
        url.push_str(sst);
    }

    // Always enable graceful shutdown to support SIGINT handling
    let (shutdown_sender, mut shutdown_receiver) = mpsc::channel(10);

    // Create ServerInfo before moving values into state
    let server_info = ServerInfo::new(port, server_token.clone(), dir.clone());

    let state = ServerState {
        dir,
        server_token,
        raw,
        source,
        sync,
        shutdown_sender: Some(shutdown_sender),
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
            "/~themes/websocket",
            get(themes::websocket_handler)
                .route_layer(middleware_fn(state.clone(), auth_middleware)),
        )
        .route(
            "/{*path}",
            get(documents::serve_path).route_layer(middleware_fn(state.clone(), auth_middleware)),
        )
        .route(
            "/",
            get(documents::serve_root).route_layer(middleware_fn(state.clone(), auth_middleware)),
        )
        .layer(create_cors_layer(cors))
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .with_state(state)
        .into_make_service();

    let listener = TcpListener::bind(&address).await?;

    if !no_startup_message {
        tracing::info!("Starting server at {url}");
    }

    // Write server info for discovery
    server_info.write()?;

    // Run server with graceful shutdown support
    let result = axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received SIGINT, stopping server gracefully");
                }
                result = shutdown_receiver.recv() => {
                    if result.is_some() {
                        tracing::debug!("Server shutdown signal received, stopping server gracefully");
                    } else {
                        tracing::warn!("Server shutdown channel closed without signal");
                    }
                }
            }
        })
        .await;

    // Cleanup server info file
    if let Err(error) = server_info.cleanup() {
        tracing::warn!("Failed to cleanup server info: {}", error);
    }

    result?;

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
    if let Some(auth_header) = headers.get("Authorization")
        && auth_header.to_str().unwrap_or_default() == ["Token ", &server_token].concat()
    {
        return Ok(next.run(request).await);
    }

    // Check if the access token is provided as a cookie
    if let Some(cookie) = cookies.get("sst")
        && cookie.value() == server_token
    {
        return Ok(next.run(request).await);
    }

    // Check if the access token is provided as a query parameter
    if let Some(token) = query.sst
        && token == *server_token
    {
        // Set the access token as a cookie. Setting path is
        // important so that the cookie is sent for all routes
        // including document websocket connections
        let mut cookie = Cookie::new("sst", token);
        cookie.set_path("/");
        cookies.add(cookie);

        return Ok(next.run(request).await);
    }

    Err((StatusCode::UNAUTHORIZED, "Unauthorized").into_response())
}

/// Create a CORS layer based on the specified level
fn create_cors_layer(level: CorsLevel) -> CorsLayer {
    match level {
        CorsLevel::None => CorsLayer::new(),
        CorsLevel::Restrictive => CorsLayer::new()
            .allow_origin(HeaderValue::from_static("http://localhost"))
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any),
        CorsLevel::Local => CorsLayer::new()
            .allow_origin([
                HeaderValue::from_static("http://localhost:3000"),
                HeaderValue::from_static("http://127.0.0.1:3000"),
                HeaderValue::from_static("http://localhost:9000"),
                HeaderValue::from_static("http://127.0.0.1:9000"),
            ])
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers(Any)
            .allow_credentials(true),
        CorsLevel::Permissive => CorsLayer::permissive(),
    }
}
