use crate::{
    config::CONFIG,
    documents::DOCUMENTS,
    jwt::{self, YEAR_SECONDS},
    projects::Projects,
    rpc::{self, Error, Response},
    utils::urls,
};
use chrono::{Duration, TimeZone, Utc};
use events::{subscribe, unsubscribe, Subscriber, SubscriptionId};
use eyre::{bail, eyre, Result};
use futures::{SinkExt, StreamExt};
use itertools::Itertools;
use jwt::JwtError;
use once_cell::sync::{Lazy, OnceCell};
use regex::{Captures, Regex};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::private::PathAsDisplay;
use tokio::sync::{mpsc, oneshot, RwLock};
use warp::{
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    ws, Filter, Reply,
};

/// Start the server
#[tracing::instrument]
pub async fn start(
    home: Option<PathBuf>,
    url: Option<String>,
    key: Option<String>,
    insecure: bool,
    traversal: bool,
    root: bool,
) -> Result<()> {
    if let Some(server) = SERVER.get() {
        let server = server.read().await;
        if server.running() {
            bail!("Server has already been started; perhaps use `stop` first");
        }
    }

    let mut server = Server::new(home, url, key, insecure, traversal, root).await?;
    server.start().await?;

    match SERVER.get() {
        Some(mutex) => {
            *(mutex.write().await) = server;
        }
        None => {
            if SERVER.set(RwLock::new(server)).is_err() {
                bail!("Unable to set server instance")
            }
        }
    }

    Ok(())
}

/// Stop the server
#[tracing::instrument]
pub async fn stop() -> Result<()> {
    if let Some(server) = SERVER.get() {
        server.write().await.stop().await?;
    } else {
        tracing::warn!("Server has not yet been started");
    }
    Ok(())
}

/// Serve a project or document
///
/// The server will be started (if it has not already been) and a URL is returned
/// containing the server's port, the path, and a token providing access to the
/// path (unless the server has no `key`). The path will be added to the server's
/// list of served paths (only paths withing these paths can be accessed).
///
/// # Arguments
///
/// - `path`: The path to be served
/// - `expiry_seconds`: The number of seconds before the token expires
/// - `single_use`: Whether the token should be single use
pub async fn serve<P: AsRef<Path>>(
    path: &P,
    expiry_seconds: Option<i64>,
    single_use: bool,
) -> Result<String> {
    let path = path.as_ref();

    // Get, or start, the server
    let server = match SERVER.get() {
        Some(server) => server,
        None => {
            let mut server = Server::new(None, None, None, false, false, false).await?;
            server.start().await?;
            SERVER.get_or_init(|| RwLock::new(server))
        }
    };
    let mut server = server.write().await;

    // Insert the path into the server's paths to allow access
    let project = Projects::project_of_path(path)?;
    server.projects.insert(project.clone());

    // If the path is in the server `home` directory, use a relative path in the URL.
    // Strip forward slashes from paths: URL paths beginning with a forward slash cause issues with 301 redirects
    // (they are confused with protocol relative URLs by the browser) and percent encoding them causes other issues
    // with cookie `Path` matching.
    let url_path = if let Ok(path) = path.strip_prefix(&server.home) {
        path
    } else if let Ok(path) = path.strip_prefix("/") {
        path
    } else {
        path
    }
    .display()
    .to_string();

    // Create a URL to path, with a path scoped token (if necessary).
    let mut url = format!("http://{}:{}/{}", server.address, server.port, url_path);
    if let Some(key) = &server.key {
        let token = jwt::encode(key, project, expiry_seconds, single_use)?;
        url += &format!("?token={}", token);
    }

    Ok(url)
}

/// The global, singleton, HTTP/WebSocket server instance
static SERVER: OnceCell<RwLock<Server>> = OnceCell::new();

/// A HTTP/WebSocket server
#[derive(Debug, Serialize)]
pub struct Server {
    /// The IP address that the server is listening on
    address: String,

    /// The port that the server is listening on
    port: u16,

    /// The secret key used to sign and verify JSON Web Tokens issued by the server
    key: Option<String>,

    /// The home project of the server
    ///
    /// This defaults to the directory that the server is started in, or if that directory
    /// is nested within a project (i.e. has a `project.json` file), the root directory of the project.
    /// Any relative paths that are requested from the server will be resolved to this
    /// directory.
    home: PathBuf,

    /// The projects that the server will allow access to
    projects: HashSet<PathBuf>,

    /// Whether traversal out of `paths` is permitted
    traversal: bool,

    /// The set of already used, single-use tokens
    used_tokens: HashSet<String>,

    /// The `oneshot` channel sender used internally to gracefully shutdown the server
    #[serde(skip)]
    shutdown_sender: Option<oneshot::Sender<()>>,
}

impl Server {
    /// Create a new server
    ///
    /// # Arguments
    ///
    /// - `home`: The root directory for files that are served (defaults to current working directory)
    /// - `url`: The URL to listen on
    /// - `key`: A secret key for signing and verifying JSON Web Tokens (defaults to random)
    /// - `insecure`: Allow unauthenticated access (i.e. no JSON Web Token)
    /// - `traversal`: Allow traversal out of the root directory is allowed
    /// - `root`: Allow serving as root user
    pub async fn new(
        home: Option<PathBuf>,
        url: Option<String>,
        key: Option<String>,
        insecure: bool,
        traversal: bool,
        root: bool,
    ) -> Result<Self> {
        let config = &CONFIG.lock().await.serve;

        let home = match &home {
            Some(home) => home.canonicalize()?,
            None => Projects::project_of_cwd()?,
        };

        let mut projects = HashSet::new();
        projects.insert(home.clone());

        let url = match url {
            Some(url) => Some(url),
            None => config.url.clone(),
        };

        let key = match key {
            Some(key) => Some(key),
            None => config.key.clone(),
        };

        let insecure = insecure || config.insecure;
        if insecure {
            tracing::warn!("Serving in insecure mode is dangerous and discouraged.")
        }

        let key = if key.is_none() {
            match insecure {
                true => None,
                false => Some(key_utils::generate()),
            }
        } else {
            key
        };

        if traversal {
            tracing::warn!("Allowing traversal out of server home directory.")
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let sudo::RunningAs::Root = sudo::check() {
            if root {
                tracing::warn!("Serving as root/administrator is dangerous and discouraged.")
            } else {
                bail!("Serving as root/administrator is not permitted by default, use the `--root` option to bypass this safety measure.")
            }
        }

        let (address, port) = match url {
            Some(url) => Self::parse_url(&url)?,
            None => ("127.0.0.1".to_string(), Self::pick_port(9000, 9011)?),
        };

        Ok(Self {
            address,
            port,
            key,
            home,
            projects,
            traversal,
            used_tokens: HashSet::new(),
            shutdown_sender: None,
        })
    }

    /// Start the server
    pub async fn start(&mut self) -> Result<()> {
        let home = self.home.clone();
        let traversal = self.traversal;

        let mut url = format!("http://{}:{}", self.address, self.port);
        if let Some(key) = &self.key {
            // Check the key is not too long
            if key.len() > 64 {
                bail!("Server key should be 64 bytes or less")
            }

            // Provide the user with a long-expiring, multiple-use token, scoped to the home project.
            let token = jwt::encode(key, home.clone(), None, false)?;
            url.push_str("?token=");
            url.push_str(&token);
        }
        tracing::info!("Serving {} at {}", home.display(), url);

        // Static files (assets embedded in binary for which authentication is not required)

        let statics = warp::get()
            .and(warp::path("~static"))
            .and(warp::path::tail())
            .and_then(get_static);

        // The following HTTP and WS endpoints all require authentication

        let authenticate = || authentication_filter(self.key.clone(), self.home.clone());

        let ws = warp::ws()
            .and(warp::path::full())
            .and(warp::query::<WsParams>())
            .and(authenticate())
            .map(ws_handshake);

        let get = warp::get()
            .and(warp::path::full())
            .and(warp::query::<GetParams>())
            .and(authenticate())
            .and(warp::any().map(move || (home.clone(), traversal)))
            .and_then(get_handler);

        // Custom `server` header
        let server_header = warp::reply::with::default_header(
            "server",
            format!(
                "Stencila/{} ({})",
                env!("CARGO_PKG_VERSION"),
                env::consts::OS
            ),
        );

        // CORS headers to allow from any origin
        let cors_headers = warp::cors()
            .allow_any_origin()
            .allow_headers(vec![
                "Content-Type",
                "Referer", // Note that this is an intentional misspelling!
                "Origin",
                "Access-Control-Allow-Origin",
            ])
            .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST])
            .max_age(24 * 60 * 60);

        let routes = statics
            .or(ws)
            .or(get)
            .with(server_header)
            .with(cors_headers)
            .recover(rejection_handler);

        // Spawn the serving task
        let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();
        let address: std::net::IpAddr = self.address.parse()?;
        let (_, future) =
            warp::serve(routes).bind_with_graceful_shutdown((address, self.port), async {
                shutdown_receiver.await.ok();
            });
        tokio::task::spawn(future);

        self.shutdown_sender = Some(shutdown_sender);

        Ok(())
    }

    /// Stop the server
    pub async fn stop(&mut self) -> Result<()> {
        tracing::debug!("Stopping server");

        CLIENTS.clear().await;

        if self.shutdown_sender.is_some() {
            // It appears to be sufficient to just set the sender to None to shutdown the server
            self.shutdown_sender = None;
            tracing::info!("Server stopped successfully");
        } else {
            tracing::info!("Server was already stopped");
        }

        Ok(())
    }

    /// Is the server running?
    pub fn running(&self) -> bool {
        self.shutdown_sender.is_some()
    }

    /// Parse a URL into address and port components
    pub fn parse_url(url: &str) -> Result<(String, u16)> {
        let url = urls::parse(url)?;
        let address = url.host().unwrap().to_string();
        let port = url
            .port_or_known_default()
            .expect("Should be a default port for the protocol");
        Ok((address, port))
    }

    /// Pick the first available port from a range, falling back to a random port
    /// if none of the ports in the range are available
    pub fn pick_port(min: u16, max: u16) -> Result<u16> {
        for port in min..max {
            if portpicker::is_free(port) {
                return Ok(port);
            }
        }
        portpicker::pick_unused_port().ok_or_else(|| eyre!("There are no free ports"))
    }
}

#[derive(Debug, Serialize)]
struct Client {
    /// The client id
    id: String,

    /// A mapping of subscription topics to subscript ids for this client
    subscriptions: HashMap<String, SubscriptionId>,

    /// The current sender for this client
    ///
    /// This is set / reset each time that the client opens
    /// a WebSocket connection
    #[serde(skip)]
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl Client {
    /// Subscribe the client to an event topic
    pub fn subscribe(&mut self, topic: &str, subscription_id: SubscriptionId) {
        self.subscriptions
            .insert(topic.to_string(), subscription_id);
    }

    /// Unsubscribe the client from an event topic
    pub fn unsubscribe(&mut self, topic: &str) -> Option<SubscriptionId> {
        self.subscriptions.remove(topic)
    }

    /// Is a client subscribed to a particular topic, or set of topics?
    pub fn subscribed(&self, topic: &str) -> bool {
        for subscription in self.subscriptions.keys() {
            if subscription == "*" || topic.starts_with(subscription) {
                return true;
            }
        }
        false
    }

    /// Send a serializable message to the client
    pub fn send(&self, message: impl Serialize) {
        match serde_json::to_string(&message) {
            Ok(json) => self.send_text(&json),
            Err(error) => tracing::error!("Error serializing to JSON `{}`", error),
        }
    }

    /// Send a text message to the client
    pub fn send_text(&self, text: &str) {
        if let Err(error) = self.sender.send(warp::ws::Message::text(text)) {
            tracing::error!("Client send error `{}`", error)
        }
    }
}

/// The global store of clients
static CLIENTS: Lazy<Clients> = Lazy::new(Clients::new);

/// A store of clients
///
/// Used to manage relaying events to clients.
#[derive(Debug)]
struct Clients {
    /// The clients
    inner: Arc<RwLock<HashMap<String, Client>>>,

    /// The sender used to subscribe to events on behalf of clients
    sender: mpsc::UnboundedSender<events::Message>,
}

impl Clients {
    /// Create a new client store and begin task for publishing events to them
    pub fn new() -> Self {
        let inner = Arc::new(RwLock::new(HashMap::new()));

        let (sender, receiver) = mpsc::unbounded_channel::<events::Message>();
        tokio::spawn(Clients::relay(inner.clone(), receiver));

        Self { inner, sender }
    }

    /// A client connected
    pub async fn connected(&self, client_id: &str, sender: mpsc::UnboundedSender<ws::Message>) {
        let mut clients = self.inner.write().await;
        match clients.entry(client_id.to_string()) {
            Entry::Occupied(mut occupied) => {
                tracing::debug!("Re-connection for client `{}`", client_id);
                let client = occupied.get_mut();
                client.sender = sender;
            }
            Entry::Vacant(vacant) => {
                tracing::debug!("New connection for client `{}`", client_id);
                vacant.insert(Client {
                    id: client_id.to_string(),
                    subscriptions: HashMap::new(),
                    sender,
                });
            }
        };
    }

    /// A client disconnected
    pub async fn disconnected(&self, client_id: &str, gracefully: bool) {
        self.remove(client_id).await;

        if gracefully {
            tracing::debug!("Graceful disconnection by client `{}`", client_id)
        } else {
            tracing::warn!("Ungraceful disconnection by client `{}`", client_id)
        }
    }

    /// Subscribe a client to an event topic
    pub async fn subscribe(&self, client_id: &str, topic: &str) {
        let mut clients = self.inner.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            tracing::debug!("Subscribing client `{}` to topic `{}`", client_id, topic);
            match subscribe(topic, Subscriber::Sender(self.sender.clone())) {
                Ok(subscription_id) => {
                    client.subscribe(topic, subscription_id);
                }
                Err(error) => {
                    tracing::error!("{}", error);
                }
            }
        } else {
            tracing::error!("No such client `{}`", client_id);
        }
    }

    /// Unsubscribe a client from an event topic
    pub async fn unsubscribe(&self, client_id: &str, topic: &str) {
        let mut clients = self.inner.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            tracing::debug!(
                "Unsubscribing client `{}` from topic `{}`",
                client_id,
                topic
            );
            if let Some(subscription_id) = client.unsubscribe(topic) {
                if let Err(error) = unsubscribe(&subscription_id) {
                    tracing::error!("{}", error);
                }
            }
        } else {
            tracing::error!("No such client `{}`", client_id);
        }
    }

    /// Remove a client from the store
    ///
    /// Removes all the client event subscriptions in addition to removing the client
    /// from the list of clients.
    pub async fn remove(&self, client_id: &str) {
        let mut clients = self.inner.write().await;
        if let Some(client) = clients.get(client_id) {
            for subscription_id in client.subscriptions.values() {
                if let Err(error) = unsubscribe(subscription_id) {
                    tracing::error!("{}", error);
                }
            }
        }
        clients.remove(client_id);
    }

    /// Remove all clients from the store
    ///
    /// Removes all clients and all their event subscriptions.
    /// This should be done when the server is stopped to avoid keeping a record
    /// of clients that have been disconnected.
    pub async fn clear(&self) {
        let mut clients = self.inner.write().await;
        for client in clients.values() {
            for subscription_id in client.subscriptions.values() {
                if let Err(error) = unsubscribe(subscription_id) {
                    tracing::error!("{}", error);
                }
            }
        }
        clients.clear();
    }

    /// Send a message to a client
    pub async fn send(&self, client_id: &str, message: impl Serialize) {
        let clients = self.inner.read().await;
        if let Some(client) = clients.get(client_id) {
            client.send(message);
        } else {
            tracing::error!("No such client `{}`", client_id);
        }
    }

    /// Relay events to clients
    ///
    /// The receiver will receive _all_ events that are published and relay them on to
    /// clients based in their subscriptions.
    async fn relay(
        clients: Arc<RwLock<HashMap<String, Client>>>,
        receiver: mpsc::UnboundedReceiver<events::Message>,
    ) {
        let mut receiver = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver);
        while let Some((topic, event)) = receiver.next().await {
            tracing::debug!("Received event for topic `{}`", topic);

            // Get a list of clients that are subscribed to this topic
            let clients = clients.read().await;
            let clients = clients
                .values()
                .filter(|client| client.subscribed(&topic))
                .collect_vec();

            // Skip this event if no one is subscribed
            if clients.is_empty() {
                continue;
            }

            // Create a JSON-RPC notification for the event and serialize it
            // so that does not need to be repeated for each client
            let params = if event.is_object() {
                serde_json::from_value(event).unwrap()
            } else {
                let mut params = HashMap::new();
                params.insert("event".to_string(), event);
                params
            };
            let notification = rpc::Notification::new(&topic, params);
            let json = match serde_json::to_string(&notification) {
                Ok(json) => json,
                Err(error) => {
                    tracing::error!("Error serializing to JSON `{}`", error);
                    continue;
                }
            };

            tracing::debug!(
                "Relaying event to subscribed clients `{}`",
                clients
                    .iter()
                    .map(|client| client.id.clone())
                    .collect::<Vec<String>>()
                    .join(",")
            );

            // Send it!
            for client in clients {
                client.send_text(&json)
            }
        }

        tracing::debug!("Relaying task ended");
    }
}

/// Return an error response
///
/// Used to have a consistent structure to error responses in the
/// handler functions below.
#[allow(clippy::unnecessary_wraps)]
fn error_response(
    code: StatusCode,
    message: &str,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({ "message": message })),
        code,
    )
    .into_response())
}

/// Static assets
///
/// During development, these are served from the `static` folder (which
/// has a symlink to `web/dist/browser` (and maybe in the future other folders).
/// At build time these are embedded in the binary. Use `include` and `exclude`
/// glob patterns to only include the assets that are required.
#[cfg(feature = "serve-http")]
#[derive(RustEmbed)]
#[folder = "static"]
#[exclude = "web/*.map"]
struct Static;

/// The version used in URL paths for static assets
/// Allows for caching control (see [`get_static`]).
const STATIC_VERSION: &str = if cfg!(debug_assertions) {
    "dev"
} else {
    env!("CARGO_PKG_VERSION")
};

/// Handle a HTTP `GET` request to the `/~static/` path
///
/// This path includes the current version number e.g. `/~static/0.127.0`. This
/// allows a `Cache-Control` header with long `max-age` and `immutable` (so that browsers do not
/// fetch / parse assets on each request) while also causing the browser cache to be busted for
/// each new version of Stencila. During development, the version is set to "dev" and the cache control
/// header is not set (for automatic reloading of re-built assets etc).
#[tracing::instrument]
async fn get_static(
    path: warp::path::Tail,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str().to_string();
    tracing::debug!("GET ~static /{}", path);

    // Remove the version number with warnings if it is not present
    // or different to current version
    let parts = path.split('/').collect_vec();
    let path = if parts.len() < 2 {
        tracing::warn!("Expected path to have at least two parts");
        path
    } else {
        let version = parts[0];
        if version != STATIC_VERSION {
            tracing::warn!(
                "Requested static assets for a version `{}` not equal to current version `{}`",
                version,
                STATIC_VERSION
            );
        }
        parts[1..].join("/")
    };

    // This is not necessary for production (since the filesystem is not touched) only
    // for development. But to keep dev and prod as consistent as possible it is
    // applied in both contexts.
    if path.contains("..") {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "Path traversal not permitted for static assets",
        );
    }

    let asset = if cfg!(debug_assertions) {
        // The `rust-embed` crate will load from the filesystem during development but
        // does not allow for symlinks (because, since https://github.com/pyros2097/rust-embed/commit/e1720ce38452c7f94d2ff32d2c120d7d427e2ebe,
        // it checks for path traversal using the canonicalized path). This is problematic for our development workflow which
        // includes live reloading of assets developed in the `web` and `components` modules. Therefore, this
        // re-implements loading of assets from the filesystem.
        let fs_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("static")
            .join(&path);
        match fs::read(&fs_path) {
            Ok(data) => data,
            Err(error) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Error reading file `{}`: {}", fs_path.display(), error),
                )
            }
        }
    } else {
        match Static::get(&path) {
            Some(asset) => asset.data.into(),
            None => {
                return error_response(
                    StatusCode::NOT_FOUND,
                    &format!("Requested static asset `{}` does not exist", &path),
                )
            }
        }
    };

    let mut response = warp::reply::Response::new(asset.into());

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );

    let cache_control = if STATIC_VERSION == "dev" {
        "no-cache"
    } else {
        "max-age=31536000, immutable"
    };
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_str(cache_control).unwrap(),
    );

    Ok(response)
}

/// Query parameters for `auth_filter`
#[derive(Deserialize)]
struct AuthParams {
    pub token: Option<String>,
}

/// A Warp filter that extracts any JSON Web Token from a `token` query parameter, `Authorization` header
/// or `token` cookie.
///
/// Returns the extracted (or refreshed) token, the claims extracted from the token and a cookie (if
/// the token did not come from a cookie originally).
fn authentication_filter(
    key: Option<String>,
    home: PathBuf,
) -> impl Filter<Extract = ((bool, String, jwt::Claims, Option<String>),), Error = warp::Rejection> + Clone
{
    warp::query::<AuthParams>()
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::cookie::optional("token"))
        .map(
            move |query: AuthParams, header: Option<String>, cookie: Option<String>| {
                (key.clone(), home.clone(), query.token, header, cookie)
            },
        )
        .and_then(
            |(key, home, param, header, cookie): (
                Option<String>,
                PathBuf,
                Option<String>,
                Option<String>,
                Option<String>,
            )| async move {
                if let Some(key) = key {
                    // Key present, so check for valid token as a query parameter, authorization header,
                    // or cookie (in that order of precedence).

                    // Attempt to get from query parameter
                    let (token, claims) = if let Some(param) = param {
                        tracing::debug!("Authentication claims from param");
                        (Some(param.clone()), jwt::decode(&param, &key))
                    } else {
                        (None, Err(JwtError::NoTokenSupplied))
                    };

                    // Attempt to get from authorization header
                    let (token, claims) = if let (Err(..), Some(header)) = (&claims, header) {
                        tracing::debug!("Authentication claims from header");
                        match jwt::from_auth_header(header) {
                            Ok(token) => (Some(token.clone()), jwt::decode(&token, &key)),
                            Err(error) => {
                                tracing::warn!("Error extracting token from header: {}", error);
                                (None, claims)
                            }
                        }
                    } else {
                        (token, claims)
                    };

                    // Attempt to get from cookie
                    let (token, claims, from_cookie) =
                        if let (Err(..), Some(cookie)) = (&claims, cookie) {
                            tracing::debug!("Authentication claims from cookie");
                            let claims = jwt::decode(&cookie, &key);
                            let ok = claims.is_ok();
                            (Some(cookie), claims, ok)
                        } else {
                            (token, claims, false)
                        };

                    // Did we get any claims from the above?
                    let claims = match claims {
                        Ok(claims) => claims,
                        Err(error) => return Err(warp::reject::custom(error)),
                    };

                    // Check for attempt to reuse a single-use token
                    if let Some(jti) = &claims.jti {
                        let server = SERVER.get().expect("Server should be instantiated");
                        if server.read().await.used_tokens.contains(jti) {
                            return Err(warp::reject::custom(JwtError::Reuse));
                        } else {
                            server.write().await.used_tokens.insert(jti.clone());
                        }
                    }

                    let project = claims.project.clone();

                    // Generate a new token if necessary (single-use or soon to expire) for use in WebSocket URLs
                    // and/or cookies.
                    let updated_token = if claims.jti.is_some()
                        || Utc.timestamp(claims.exp, 0) < Utc::now() + Duration::seconds(60)
                    {
                        jwt::encode(&key, project.clone(), Some(YEAR_SECONDS), false)
                            .expect("Should encode")
                    } else {
                        token.clone().unwrap_or_default()
                    };

                    // Provide a token cookie if the claims did not come from a cookie or if it
                    // has been refreshed
                    // Token expires at the end of the browser session and should only be sent to
                    // URL paths that are within the project.
                    let cookie = if !from_cookie || updated_token != token.unwrap_or_default() {
                        // If the project is within the home project (ie. a subfolder) then need to strip the prefix
                        let path = if project == home {
                            PathBuf::from("/".to_string())
                        } else if let Ok(rest) = project.strip_prefix(home) {
                            rest.to_path_buf()
                        } else {
                            project
                        }
                        .display()
                        .to_string();
                        Some(format!(
                            "token={}; Path={}; SameSite; HttpOnly",
                            updated_token, path
                        ))
                    } else {
                        None
                    };

                    Ok((true, updated_token, claims, cookie))
                } else {
                    // No key, so in insecure mode. Return empty token and default claims (they won't be used anyway) and no cookie.
                    Ok((false, "".to_string(), jwt::Claims::default(), None))
                }
            },
        )
}

/// Query parameters for `get_handler`
#[derive(Debug, Deserialize)]
struct GetParams {
    /// The mode "read", "view", "exec", or "edit"
    mode: Option<String>,

    /// The format to view or edit
    format: Option<String>,

    /// The theme (when format is `html`)
    theme: Option<String>,

    /// Should web components be loaded
    components: Option<String>,

    /// An authentication token
    ///
    /// Only used here only to determine whether to redirect (but used in `authentication_filter`
    /// for actual authentication).
    token: Option<String>,
}

/// Handle a HTTP `GET` request for a file or directory
#[tracing::instrument(skip(cookie))]
async fn get_handler(
    path: warp::path::FullPath,
    params: GetParams,
    (secure, token, claims, cookie): (bool, String, jwt::Claims, Option<String>),
    (home, traversal): (PathBuf, bool),
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str();
    tracing::debug!("GET {}", path);

    // Determine if the requested path is relative to the server `home` directory;
    // otherwise construct an absolute path accordingly
    let fs_path = path.strip_prefix('/').unwrap_or(path).to_string();
    let fs_path = urlencoding::decode(&fs_path)
        .map_or_else(|_| fs_path.clone(), |fs_path| fs_path.to_string());
    let fs_path = Path::new(&fs_path);
    let fs_path = if let Ok(path) = home.join(fs_path).canonicalize() {
        // Path found in home directory
        path
    } else if let Ok(path) = fs_path.canonicalize() {
        // Path found elsewhere on the filesystem
        path
    } else if let Ok(path) = PathBuf::from("/").join(fs_path).canonicalize() {
        // Path found elsewhere on the filesystem (when stripped leading slash is added back; see `serve()`)
        path
    } else {
        return error_response(
            StatusCode::NOT_FOUND,
            &format!("Requested path `{}` does not exist", fs_path.display()),
        );
    }
    .to_path_buf();

    // Check the path is within one of the server's `projects`
    // Because `projects` may be appended to at runtime, it is necessary to get these
    // from the server instance.
    if !traversal {
        let server = SERVER
            .get()
            .expect("Server should be instantiated")
            .read()
            .await;

        let mut ok = false;
        for project in &server.projects {
            if fs_path.strip_prefix(&project).is_ok() {
                ok = true;
                break;
            }
        }
        if !ok {
            return error_response(
                StatusCode::FORBIDDEN,
                "Traversal outside of server's home, or registered, projects is not permitted",
            );
        }
    }

    // Check the path is within the project for which authorization is given
    if secure && fs_path.strip_prefix(&claims.project).is_err() {
        return error_response(
            StatusCode::FORBIDDEN,
            "Insufficient permissions to access this directory or file",
        );
    }

    let format = params.format.unwrap_or_else(|| "html".into());
    let mode = params.mode.unwrap_or_else(|| "view".into());
    let theme = params.theme.unwrap_or_else(|| "wilmore".into());
    let components = params.components.unwrap_or_else(|| "static".into());

    let (content, mime, redirect) = if params.token.is_some() && claims.jti.is_some() {
        // A token is in the URL. For address bar aesthetics, and to avoid re-use on page refresh, if the token is
        // single-use (has a `jti` claim), redirect to a token-less URL. Note that we set a token cookie
        // below to replace the URL-based token.
        (
            html_page_redirect(path).as_bytes().to_vec(),
            "text/html".to_string(),
            true,
        )
    } else if fs_path.is_dir() {
        // Request for a path that is a folder. Return a listing
        (
            html_directory_listing(&home, &fs_path).as_bytes().to_vec(),
            "text/html".to_string(),
            false,
        )
    } else if format == "raw" {
        // Request for raw content of the file (e.g. an image within the HTML encoding of a
        // Markdown document)
        let content = match fs::read(&fs_path) {
            Ok(content) => content,
            Err(error) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("When reading file `{}`", error),
                )
            }
        };

        let mime = mime_guess::from_path(fs_path).first_or_octet_stream();

        (content, mime.to_string(), false)
    } else {
        // Request for a document in some format (usually HTML)
        match DOCUMENTS.open(&fs_path, None).await {
            Ok(document) => {
                let document = DOCUMENTS.get(&document.id).await.unwrap();
                let document = document.lock().await;
                let content = match document.dump(Some(format.clone())).await {
                    Ok(content) => content,
                    Err(error) => {
                        return error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            &format!("While converting document to {} `{}`", format, error),
                        )
                    }
                };

                let content = match format.as_str() {
                    "html" => {
                        let project =
                            Projects::project_of_path(&fs_path).unwrap_or_else(|_| fs_path.clone());

                        let project = if let Ok(project) = project.strip_prefix("/") {
                            project.display()
                        } else {
                            project.display()
                        }
                        .to_string();

                        html_rewrite(
                            &content,
                            &mode,
                            &theme,
                            &components,
                            &token,
                            &project,
                            &home,
                            &fs_path,
                        )
                    }
                    _ => content,
                }
                .as_bytes()
                .to_vec();

                let mime = mime_guess::from_ext(&format).first_or_octet_stream();

                (content, mime.to_string(), false)
            }
            Err(error) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("While opening document `{}`", error),
                )
            }
        }
    };

    let mut response = warp::reply::Response::new(content.into());

    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_str(&mime).unwrap());

    if redirect {
        *response.status_mut() = StatusCode::MOVED_PERMANENTLY;
        response
            .headers_mut()
            .insert(header::LOCATION, HeaderValue::from_str(path).unwrap());
    }

    if let Some(cookie) = cookie {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    }

    Ok(response)
}

/// Generate HTML for a page redirect
///
/// Although the MOVED_PERMANENTLY status code should trigger the redirect, this
/// provides HTML / JavaScript fallbacks.
fn html_page_redirect(path: &str) -> String {
    format!(
        r#"<!DOCTYPE HTML>
<html lang="en-US">
<head>
    <title>Redirecting</title>
    <meta charset="UTF-8">
    <meta http-equiv="refresh" content="0; url={}">
    <script type="text/javascript">window.location.href = "{}"</script>
</head>
<body>If you are not redirected automatically, please follow this <a href="{}">link</a>.</body>
</html>"#,
        path, path, path
    )
}

/// Generate HTML for a directory listing
///
/// Note: If the `dir` is outside of `home` (i.e. traversal was allowed) then
/// no entries will be shown.
fn html_directory_listing(home: &Path, dir: &Path) -> String {
    let entries = match dir.read_dir() {
        Ok(entries) => entries,
        Err(error) => {
            // This should be an uncommon error but to avoid an unwrap...
            tracing::error!("{}", error);
            return "<p>Something went wrong</p>".to_string();
        }
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();

            let href = match path.strip_prefix(home) {
                Ok(href) => href,
                Err(..) => return None,
            };

            let name = match path.strip_prefix(dir) {
                Ok(name) => name,
                Err(..) => return None,
            };

            Some(format!(
                "<p><a href=\"/{}\">{}</a></p>",
                href.display(),
                name.display()
            ))
        })
        .collect::<Vec<String>>()
        .concat()
}

/// Rewrite HTML to serve local files and wrap with desired theme etc.
///
/// Only local files somewhere withing the current working directory are
/// served.
#[allow(clippy::too_many_arguments)]
pub fn html_rewrite(
    body: &str,
    mode: &str,
    theme: &str,
    components: &str,
    token: &str,
    project: &str,
    home: &Path,
    document: &Path,
) -> String {
    let static_root = ["/~static/", STATIC_VERSION].concat();

    // Head element for theme
    let themes = format!(
        r#"<link href="{static_root}/themes/themes/{theme}/styles.css" rel="stylesheet">"#,
        static_root = static_root,
        theme = theme
    );

    // Head elements for web client
    let web = format!(
        r#"
    <link href="{static_root}/web/{mode}.css" rel="stylesheet">
    <script src="{static_root}/web/{mode}.js"></script>
    <script>
        const startup = stencilaWebClient.main("{client}", "{project}", "{snapshot}", "{document}", null, "{token}");
        startup().catch((err) => console.error('Error during startup', err))
    </script>"#,
        static_root = static_root,
        mode = mode,
        client = uuids::generate("cl"),
        token = token,
        project = project,
        snapshot = "current",
        document = document.as_display().to_string()
    );

    // Head elements for web components
    let components = match components {
        "none" => "".to_string(),
        _ => {
            let base = match components {
                "remote" => {
                    "https://unpkg.com/@stencila/components/dist/stencila-components".to_string()
                }
                _ => [&static_root, "/components"].concat(),
            };
            format!(
                r#"
                <script src="{}/stencila-components.esm.js" type="module"> </script>
                <script src="{}/stencila-components.js" type="text/javascript" nomodule=""> </script>
                "#,
                base, base
            )
        }
    };

    // Rewrite body content so that links to files work
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#""file://(.*?)""#).expect("Unable to create regex"));

    let body = REGEX.replace_all(body, |captures: &Captures| {
        let path = captures
            .get(1)
            .expect("Should always have first capture")
            .as_str();
        let path = match Path::new(path).canonicalize() {
            Ok(path) => path,
            // Redact the path if it can not be canonicalized
            Err(_) => return "\"\"".to_string(),
        };
        match path.strip_prefix(home) {
            Ok(path) => ["\"/", &path.display().to_string(), "?format=raw\""].concat(),
            // Redact the path if it is outside of the current directory
            Err(_) => "\"\"".to_string(),
        }
    });

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        {themes}
        {web}
        {components}
    </head>
    <body>
        {body}
    </body>
</html>"#,
        themes = themes,
        web = web,
        components = components,
        body = body
    )
}

/// Parameters for the WebSocket handshake
#[derive(Debug, Deserialize)]
struct WsParams {
    /// The id of the client
    client: String,
}

/// Perform a WebSocket handshake / upgrade
///
/// This function is called at the start of a WebSocket connection.
/// Each WebSocket connection is authorized to access a single project.
/// Authorization is done by checking the `project` in the JWT claims
/// against the requested path.
#[tracing::instrument(skip(_cookie))]
fn ws_handshake(
    ws: warp::ws::Ws,
    path: warp::path::FullPath,
    params: WsParams,
    (secure, token, claims, _cookie): (bool, String, jwt::Claims, Option<String>),
) -> Box<dyn warp::Reply> {
    use warp::reply;

    tracing::debug!("WebSocket handshake");

    // Check that client is authorized to access the path
    // On MacOS and Linux the leading slash is removed from the URL path so it
    // is necessary to check against both the path, and the path less any leading slash.

    let project = claims.project.display().to_string();

    let path = path.as_str();
    let fs_path = path.strip_prefix('/').unwrap_or(path).to_string();
    let fs_path = urlencoding::decode(&fs_path)
        .map_or_else(|_| fs_path.clone(), |fs_path| fs_path.to_string());

    if secure && project != fs_path && project != ["/", &fs_path].concat() {
        return Box::new(reply::with_status(reply(), StatusCode::UNAUTHORIZED));
    }

    Box::new(ws.on_upgrade(|socket| ws_connected(socket, params.client)))
}

/// Handle a WebSocket connection
///
/// This function is called after the handshake, when a WebSocket client
/// has successfully connected.
#[tracing::instrument(skip(socket))]
async fn ws_connected(socket: warp::ws::WebSocket, client_id: String) {
    tracing::debug!("WebSocket connected");

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the client's websocket.
    let (client_sender, client_receiver) = mpsc::unbounded_channel();
    let mut client_receiver = tokio_stream::wrappers::UnboundedReceiverStream::new(client_receiver);

    let client_clone = client_id.clone();
    tokio::task::spawn(async move {
        while let Some(message) = client_receiver.next().await {
            if let Err(error) = ws_sender.send(message).await {
                let message = error.to_string();
                if message == "Connection closed normally" {
                    CLIENTS.disconnected(&client_clone, true).await
                } else {
                    tracing::error!("Websocket send error `{}`", error);
                }
            }
        }
    });

    // Save / update the client
    CLIENTS.connected(&client_id, client_sender).await;

    while let Some(result) = ws_receiver.next().await {
        // Get the message
        let message = match result {
            Ok(message) => message,
            Err(error) => {
                let message = error.to_string();
                if message == "WebSocket protocol error: Connection reset without closing handshake"
                {
                    CLIENTS.disconnected(&client_id, false).await
                } else {
                    tracing::error!("Websocket receive error `{}`", error);
                }
                continue;
            }
        };

        // Parse the message as a string, skipping non-text messages
        let json = if let Ok(string) = message.to_str() {
            string
        } else {
            continue;
        };

        // Parse the message, returning an error to the client if that fails
        let request = match serde_json::from_str::<rpc::Request>(json) {
            Ok(request) => request,
            Err(error) => {
                let error = rpc::Error::parse_error(&error.to_string());
                let response = rpc::Response::new(None, None, Some(error));
                CLIENTS.send(&client_id, response).await;
                continue;
            }
        };

        // Dispatch the request and send back the response and update subscriptions
        let (response, subscription) = request.dispatch(&client_id).await;
        CLIENTS.send(&client_id, response).await;
        match subscription {
            rpc::Subscription::Subscribe(topic) => {
                CLIENTS.subscribe(&client_id, &topic).await;
            }
            rpc::Subscription::Unsubscribe(topic) => {
                CLIENTS.unsubscribe(&client_id, &topic).await;
            }
            rpc::Subscription::None => (),
        }
    }

    // Record that the client has disconnected gracefully
    CLIENTS.disconnected(&client_id, true).await
}

/// Handle a rejection by converting into a JSON-RPC response
///
/// The above handlers can not handle all errors, in particular, they do not
/// handle JSON parsing errors (which are rejected by the `warp::body::json` filter).
/// This therefore ensures that any request expecting a JSON-RPC response, will get
/// a JSON-RPC response (in these cases containing and error code and message).
#[tracing::instrument]
async fn rejection_handler(
    rejection: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let error = if let Some(error) = rejection.find::<jwt::JwtError>() {
        Error::invalid_request_error(&format!("{}", error))
    } else if let Some(error) = rejection.find::<warp::filters::body::BodyDeserializeError>() {
        Error::invalid_request_error(&format!("{}", error))
    } else if rejection.find::<warp::reject::MethodNotAllowed>().is_some() {
        Error::invalid_request_error("Invalid HTTP method and/or path")
    } else {
        Error::server_error("Unknown error")
    };

    tracing::error!("{}", error);

    Ok(warp::reply::with_status(
        warp::reply::json(&Response {
            error: Some(error),
            ..Default::default()
        }),
        StatusCode::BAD_REQUEST,
    ))
}

pub mod config {
    use defaults::Defaults;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_with::skip_serializing_none;
    use validator::Validate;

    /// Server
    ///
    /// Configuration settings for running as a server
    #[skip_serializing_none]
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default)]
    #[schemars(deny_unknown_fields)]
    pub struct ServeConfig {
        /// The URL to serve on
        #[validate(url(message = "Not a valid URL"))]
        pub url: Option<String>,

        /// Secret key to use for signing and verifying JSON Web Tokens
        pub key: Option<String>,

        /// Do not require a JSON Web Token to access the server
        #[def = "false"]
        pub insecure: bool,
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use std::path::PathBuf;

    use super::*;
    use async_trait::async_trait;
    use cli_utils::{result, Result, Run};
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage the HTTP/WebSocket server",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Command {
        #[structopt(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub enum Action {
        Start(Start),
        Stop(Stop),
        Show(Show),
        Clients(Clients),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::Start(action) => action.run().await,
                Action::Stop(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Clients(action) => action.run().await,
            }
        }
    }

    /// Start the server
    ///
    /// ## Ports and addresses
    ///
    /// Use the <url> argument to change the port and/or address that the server
    /// listens on. This argument can be a partial, or complete, URL.
    ///
    /// For example, to serve on port 8000 instead of the default port,
    ///
    ///    stencila server start :8000
    ///
    /// To serve on all IPv4 addresses on the machine, instead of only `127.0.0.1`,
    ///
    ///    stencila server start 0.0.0.0
    ///
    /// Or if you prefer, use a complete URL including the scheme e.g.
    ///
    ///   stencila server start http://127.0.0.1:9000
    ///
    /// ## Security
    ///
    /// By default, the server requires authentication using JSON Web Token. A token is
    /// printed as part of the server's URL at startup. To turn authorization off, for example
    /// if you are using some other authentication layer in front of the server, use the `--insecure`
    /// flag.
    ///
    /// By default, this command will NOT run as a root (Linux/Mac OS/Unix) or administrator (Windows) user.
    /// Use the `--root` option, with extreme caution, to allow to be run as root.
    ///
    /// Most of these options can be set in the Stencila configuration file. See `stencila config get serve`
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Start {
        /// The home directory for the server to serve from
        ///
        /// Defaults to the current directory or an ancestor project directory (if the current directory
        /// is within a project).
        home: Option<PathBuf>,

        /// The URL to serve on
        ///
        /// Defaults to the `STENCILA_URL` environment variable, the value set in config
        /// or otherwise `http://127.0.0.1:9000`.
        #[structopt(short, long, env = "STENCILA_URL")]
        url: Option<String>,

        /// Secret key to use for signing and verifying JSON Web Tokens
        ///
        /// Defaults to the `STENCILA_KEY` environment variable, the value set in config
        /// or otherwise a randomly generated value.
        #[structopt(short, long, env = "STENCILA_KEY")]
        key: Option<String>,

        /// Do not require a JSON Web Token to access the server
        ///
        /// For security reasons (any client can access files and execute code) this should be avoided.
        #[structopt(long)]
        insecure: bool,

        /// Allow traversal out of the server's home directory
        ///
        /// For security reasons (clients can access any file on the filesystem) this should be avoided.
        #[structopt(long)]
        traversal: bool,

        /// Allow root (Linux/Mac OS/Unix) or administrator (Windows) user to serve
        ///
        /// For security reasons (clients may be able to execute code as root) this should be avoided.
        #[structopt(long)]
        root: bool,
    }
    #[async_trait]
    impl Run for Start {
        async fn run(&self) -> Result {
            if self.key.is_some() {
                tracing::warn!("Server key set on command line could be sniffed by malicious processes; prefer to set it in config file.");
            };

            start(
                self.home.clone(),
                self.url.clone(),
                self.key.clone(),
                self.insecure,
                self.traversal,
                self.root,
            )
            .await?;

            // If not in interactive mode then just sleep here forever to avoid finishing
            if std::env::var("STENCILA_INTERACT_MODE").is_err() {
                use tokio::time::{sleep, Duration};
                sleep(Duration::MAX).await;
            }

            result::nothing()
        }
    }

    /// Stop the server
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Stop {}
    #[async_trait]
    impl Run for Stop {
        async fn run(&self) -> Result {
            stop().await?;

            result::nothing()
        }
    }

    /// Show details of the server
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {}
    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            match SERVER.get() {
                Some(server) => {
                    let server = server.read().await;
                    result::value(&*server)
                }
                None => {
                    tracing::info!("No server currently running");
                    result::nothing()
                }
            }
        }
    }

    /// List the clients connected to the server
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Clients {}
    #[async_trait]
    impl Run for Clients {
        async fn run(&self) -> Result {
            let clients = CLIENTS.inner.read().await;
            result::value(&*clients)
        }
    }
}
