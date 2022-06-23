use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use jwt::JwtError;
use warp::{
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    ws, Filter, Reply,
};

use common::{
    chrono::{Duration, TimeZone, Utc},
    eyre::{bail, eyre, Result},
    futures::{SinkExt, StreamExt},
    itertools::Itertools,
    once_cell::sync::{Lazy, OnceCell},
    regex::{Captures, Regex},
    serde::{Deserialize, Serialize},
    serde_json,
    tokio::{
        self,
        sync::{mpsc, RwLock},
        task::JoinHandle,
    },
    tracing,
};
use events::{subscribe, unsubscribe, Subscriber, SubscriptionId};
use http_utils::{http, urlencoding};
use server_next::statics::{get_static_parts, STATIC_VERSION};
use stencila_schema::Node;
use uuids::generate;

use crate::{
    config::CONFIG,
    documents::DOCUMENTS,
    jwt::{self, YEAR_SECONDS},
    projects::Projects,
    rpc::{self, Error, Response},
    utils::urls,
};

/// Main server entry point function
///
/// Used when there is no fancy CLI and all that is wanted is a minimal
/// server process.
///
/// This is currently bare bones and quite useless because it uses an unknown, randomly
/// generated secret key for auth. Instead, it should read options from config file,
/// start accordingly, initialize logging etc and gracefully shutdown on `SIGINT`.
pub async fn main() -> Result<()> {
    use tokio::time::{sleep, Duration};

    start(None, None, None, false, false, false, None, None, false).await?;
    sleep(Duration::MAX).await;

    Ok(())
}

/// Start the server
#[allow(clippy::too_many_arguments)]
#[tracing::instrument]
pub async fn start(
    home: Option<PathBuf>,
    url: Option<String>,
    key: Option<String>,
    insecure: bool,
    traversal: bool,
    root: bool,
    max_inactivity: Option<u64>,
    max_duration: Option<u64>,
    log_requests: bool,
) -> Result<JoinHandle<()>> {
    if let Some(server) = SERVER.get() {
        let server = server.read().await;
        if server.running() {
            bail!("Server has already been started; perhaps use `stop` first");
        }
    }

    let mut server = Server::new(
        home,
        url,
        key,
        insecure,
        traversal,
        root,
        max_inactivity,
        max_duration,
        log_requests,
    )
    .await?;
    let join_handle = server.start().await?;

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

    Ok(join_handle)
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

/// The time that the server was started
static START_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// The time that the last request activity was recorded (e.g. a HTTP or RPC request)
static ACTIVITY_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// Get the current timestamp in seconds
fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Record activity
///
/// Note: this is separate to metric recording because some requests (e.g. to `~metrics` or `~static`
/// routes) should not be recorded as activity
fn record_activity() {
    ACTIVITY_TIMESTAMP.fetch_max(timestamp(), Ordering::Relaxed);
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

    // Insert the path into the server's paths to allow access
    let project = Projects::project_of_path(path)?;

    // Get, or start, the server
    let server = match SERVER.get() {
        Some(server) => server,
        None => {
            let mut server = Server::new(
                Some(project.clone()),
                None,
                None,
                false,
                false,
                false,
                None,
                None,
                false,
            )
            .await?;
            server.start().await?;
            SERVER.get_or_init(|| RwLock::new(server))
        }
    };
    let server = server.read().await;

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
        let token = jwt::encode(key, Some(project), expiry_seconds, single_use)?;
        url += &format!("?token={}", token);
    }

    Ok(url)
}

/// The global, singleton, HTTP/WebSocket server instance
static SERVER: OnceCell<RwLock<Server>> = OnceCell::new();

/// A HTTP/WebSocket server
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
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

    /// Whether traversal out of `paths` is permitted
    traversal: bool,

    /// Maximum number of seconds of inactivity before the server shutsdown
    max_inactivity: u64,

    /// Maximum number of seconds running before the server shutsdown
    max_duration: u64,

    /// Whether each request should be logged
    log_requests: bool,

    /// The set of already used, single-use tokens
    used_tokens: HashSet<String>,

    /// The `mpsc` channel sender used internally to gracefully shutdown the server
    #[serde(skip)]
    shutdown_sender: Option<mpsc::Sender<()>>,
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
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        home: Option<PathBuf>,
        url: Option<String>,
        key: Option<String>,
        insecure: bool,
        traversal: bool,
        root: bool,
        max_inactivity: Option<u64>,
        max_duration: Option<u64>,
        log_requests: bool,
    ) -> Result<Self> {
        let config = &CONFIG.lock().await.server;

        let home = match &home {
            Some(home) => home.canonicalize()?,
            None => Projects::project_of_cwd()?,
        };

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
                false => Some(key_utils::generate("ssk")),
            }
        } else {
            key
        };

        if traversal {
            tracing::warn!("Allowing traversal out of server home directory.")
        }

        if root {
            tracing::warn!("Serving as root/administrator is dangerous and discouraged.")
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let sudo::RunningAs::Root = sudo::check() {
            if !root {
                bail!("Serving as root/administrator is not permitted by default, use the `--root` option to bypass this safety measure.")
            }
        }

        let (address, port) = match url {
            Some(url) => Self::parse_url(&url)?,
            None => ("127.0.0.1".to_string(), Self::pick_port(9000, 9011)?),
        };

        let ten_years = 315360000;
        let max_inactivity = max_inactivity.unwrap_or(ten_years);
        let max_duration = max_duration.unwrap_or(ten_years);

        Ok(Self {
            address,
            port,
            key,
            home,
            traversal,
            max_inactivity,
            max_duration,
            log_requests,
            used_tokens: HashSet::new(),
            shutdown_sender: None,
        })
    }

    /// Start the server
    pub async fn start(&mut self) -> Result<JoinHandle<()>> {
        let home = self.home.clone();
        let traversal = self.traversal;

        let mut url = format!(
            "http://{}:{}",
            self.address.replace("0.0.0.0", "127.0.0.1"),
            self.port
        );
        if let Some(key) = &self.key {
            // Check the key is not too long
            if key.len() > 64 {
                bail!("Server key should be 64 bytes or less")
            }

            // Provide the user with a long-expiring, multiple-use token, scoped to the home project.
            let token = jwt::encode(key, Some(home.clone()), None, false)?;
            url.push_str("?token=");
            url.push_str(&token);
        }
        tracing::info!("Serving {} at {}", home.display(), url);

        // HTTP endpoints without authentication

        let statics = warp::get()
            .and(warp::path("~static"))
            .and(warp::path::tail())
            .and_then(get_static);

        let metrics = warp::get()
            .and(warp::path("~metrics"))
            .and_then(get_metrics);

        let hooks = warp::post()
            .and(warp::path("~hooks"))
            .and(warp::query::<HooksParams>())
            .and(warp::header::headers_cloned())
            .and(warp::body::content_length_limit(10 * 1_048_576)) // 10MB limit
            .and(warp::body::json())
            .and_then(post_hooks);

        // HTTP and WS endpoints requiring authentication

        let authenticate = || authentication_filter(self.key.clone(), self.home.clone());

        let terminal = warp::get()
            .and(warp::path("~terminal"))
            .and(authenticate())
            .and_then(terminal_handler);

        let attach = warp::path("~attach")
            .and(warp::ws())
            .and(authenticate())
            .map(attach_handler);

        let rpc_ws = warp::path("~rpc")
            .and(warp::ws())
            .and(warp::query::<WsParams>())
            .and(authenticate())
            .map(rpc_ws_handler);

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

        let log_requests = self.log_requests;
        let log = warp::log::custom(move |info| {
            if !log_requests {
                return;
            }

            let method = info.method().as_str();
            let path = info.path();
            let time = info.elapsed();
            let status = info.status().as_u16();

            macro_rules! event {
                ($level:expr) => {
                    tracing::event!(
                        $level,
                        method,
                        path,
                        status,
                        time = time.as_micros() as u64,
                        referer = info.referer(),
                        user_agent = info.user_agent(),
                        "{} {} {} {:?}",
                        method,
                        path,
                        status,
                        time,
                    );
                };
            }

            use tracing::Level;
            if status < 400 {
                event!(Level::INFO);
            } else if status < 500 {
                event!(Level::WARN);
            } else {
                event!(Level::ERROR);
            }
        });

        let routes = statics
            .or(metrics)
            .or(hooks)
            .or(terminal)
            .or(attach)
            .or(rpc_ws)
            .or(get)
            .with(server_header)
            .with(cors_headers)
            .with(log)
            .recover(rejection_handler);

        // Spawn the serving task
        let (shutdown_sender, mut shutdown_receiver) = mpsc::channel::<()>(1);
        let address: std::net::IpAddr = self.address.parse()?;
        let (_, future) =
            warp::serve(routes).bind_with_graceful_shutdown((address, self.port), async move {
                shutdown_receiver.recv().await;
            });
        let serve_task = tokio::task::spawn(future);

        // Initialize timestamps and pawn a timing task to shutdown the server after inactivity,
        // or a maximum duration
        START_TIMESTAMP.fetch_max(timestamp(), Ordering::SeqCst);
        ACTIVITY_TIMESTAMP.fetch_max(timestamp(), Ordering::SeqCst);
        let shutdown_sender_clone = shutdown_sender.clone();
        let max_duration = self.max_duration;
        let max_inactivity = self.max_inactivity;
        tokio::task::spawn(async move {
            use tokio::time::{sleep, Duration};
            loop {
                let now = timestamp();

                let inactivity_remaining = max_inactivity
                    .saturating_sub(now.saturating_sub(ACTIVITY_TIMESTAMP.load(Ordering::SeqCst)));
                let duration_remaining = max_duration
                    .saturating_sub(now.saturating_sub(START_TIMESTAMP.load(Ordering::SeqCst)));
                if inactivity_remaining == 0 || duration_remaining == 0 {
                    if inactivity_remaining == 0 {
                        tracing::info!(
                            "Server shutting down after maximum period of inactivity of {} seconds",
                            max_inactivity
                        );
                    } else {
                        tracing::info!(
                            "Server shutting down after maximum duration of {} seconds",
                            max_duration
                        );
                    }
                    shutdown_sender_clone.send(()).await.ok();
                    return;
                }

                SECONDS_TO_SHUTDOWN.set(
                    std::cmp::min(inactivity_remaining, duration_remaining)
                        .try_into()
                        .unwrap_or(i64::MAX),
                );

                sleep(Duration::from_secs(1)).await;
            }
        });

        self.shutdown_sender = Some(shutdown_sender);

        Ok(serve_task)
    }

    /// Stop the server
    pub async fn stop(&mut self) -> Result<()> {
        tracing::debug!("Stopping server");

        WEBSOCKET_CLIENTS.clear().await;

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

static HTTP_REQUESTS_COUNT: Lazy<prometheus::IntCounterVec> = Lazy::new(|| {
    prometheus::IntCounterVec::new(
        prometheus::Opts::new(
            "stencila_http_requests_count",
            "Count of HTTP requests by method and path",
        ),
        &["method", "path"],
    )
    .expect("Unable to create metric")
});

static RPC_REQUESTS_COUNT: Lazy<prometheus::IntCounterVec> = Lazy::new(|| {
    prometheus::IntCounterVec::new(
        prometheus::Opts::new(
            "stencila_rpc_requests_count",
            "Count of RPC requests by method",
        ),
        &["method"],
    )
    .expect("Unable to create metric")
});

static WEBSOCKET_CLIENTS_COUNT: Lazy<prometheus::IntGauge> = Lazy::new(|| {
    prometheus::IntGauge::new(
        "stencila_websocket_clients",
        "Count of Websocket clients currently connected",
    )
    .expect("Unable to create metric")
});

static SECONDS_TO_SHUTDOWN: Lazy<prometheus::IntGauge> = Lazy::new(|| {
    prometheus::IntGauge::new(
        "stencila_seconds_to_shutdown",
        "Number of seconds until the server shutsdown",
    )
    .expect("Unable to create metric")
});

static METRICS_REGISTRY: Lazy<prometheus::Registry> = Lazy::new(|| {
    let registry = prometheus::Registry::new();

    registry
        .register(Box::new(HTTP_REQUESTS_COUNT.clone()))
        .expect("Unable to register metric");

    registry
        .register(Box::new(RPC_REQUESTS_COUNT.clone()))
        .expect("Unable to register metric");

    registry
        .register(Box::new(WEBSOCKET_CLIENTS_COUNT.clone()))
        .expect("Unable to register metric");

    registry
        .register(Box::new(SECONDS_TO_SHUTDOWN.clone()))
        .expect("Unable to register metric");

    registry
});

fn record_http_request(method: &str, path: &str) {
    HTTP_REQUESTS_COUNT.with_label_values(&[method, path]).inc();
}

fn record_rpc_request(method: &str) {
    RPC_REQUESTS_COUNT.with_label_values(&[method]).inc();
}

#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
struct WebsocketClient {
    /// The client id
    id: String,

    /// The event topics that this client is subscribed to
    subscriptions: HashSet<String>,

    /// The current sender for this client
    ///
    /// This is set / reset each time that the client opens
    /// a WebSocket connection
    #[serde(skip)]
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl WebsocketClient {
    /// Subscribe the client to an event topic
    pub fn subscribe(&mut self, topic: &str) {
        self.subscriptions.insert(topic.to_string());
    }

    /// Unsubscribe the client from an event topic
    pub fn unsubscribe(&mut self, topic: &str) {
        self.subscriptions.remove(topic);
    }

    /// Is a client subscribed to a particular topic, or set of topics?
    pub fn subscribed(&self, topic: &str) -> bool {
        for subscription in &self.subscriptions {
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
            tracing::error!("Websocket client send error `{}`", error)
        }
    }
}

/// The global store of Websocket clients
static WEBSOCKET_CLIENTS: Lazy<WebsocketClients> = Lazy::new(WebsocketClients::new);

/// A store of clients
///
/// Used to manage relaying events to clients.
#[derive(Debug)]
struct WebsocketClients {
    /// The clients
    inner: Arc<RwLock<HashMap<String, WebsocketClient>>>,

    /// The event subscriptions held on behalf of clients
    ///
    /// Used to keep track of the number of clients subscribed to each topic.
    /// This ensures that we don't subscribe to the same event more than once (which results in
    /// the same event being relayed to each client more than once) and that we can unsubscribe when
    /// it becomes zero.
    subscriptions: Arc<RwLock<HashMap<String, (SubscriptionId, usize)>>>,

    /// The sender used to subscribe to events on behalf of clients
    sender: mpsc::UnboundedSender<events::Message>,
}

impl WebsocketClients {
    /// Create a new client store and begin task for publishing events to them
    pub fn new() -> Self {
        let inner = Arc::new(RwLock::new(HashMap::new()));

        let subscriptions = Arc::new(RwLock::new(HashMap::new()));

        let (sender, receiver) = mpsc::unbounded_channel::<events::Message>();
        tokio::spawn(WebsocketClients::relay(inner.clone(), receiver));

        Self::ping(inner.clone());

        Self {
            inner,
            subscriptions,
            sender,
        }
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
                vacant.insert(WebsocketClient {
                    id: client_id.to_string(),
                    subscriptions: HashSet::new(),
                    sender,
                });
            }
        };
    }

    /// A client disconnected
    pub async fn disconnected(&self, client_id: &str, gracefully: bool) {
        self.remove(client_id).await;

        if gracefully {
            tracing::trace!("Graceful disconnect by client `{}`", client_id)
        } else {
            tracing::warn!("Ungraceful disconnect by client `{}`", client_id)
        }
    }

    /// Subscribe a client to an event topic
    pub async fn subscribe(&self, client_id: &str, topic: &str) {
        let mut clients = self.inner.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            tracing::trace!("Subscribing client `{}` to topic `{}`", client_id, topic);
            let mut subscriptions = self.subscriptions.write().await;
            match subscriptions.entry(topic.to_string()) {
                Entry::Occupied(mut occupied) => {
                    occupied.get_mut().1 += 1;
                }
                Entry::Vacant(vacant) => {
                    match subscribe(topic, Subscriber::UnboundedSender(self.sender.clone())) {
                        Ok(subscription_id) => {
                            vacant.insert((subscription_id, 1));
                        }
                        Err(error) => {
                            tracing::error!(
                                "While attempting to subscribe to event topic `{}`:",
                                error
                            );
                        }
                    }
                }
            }
            client.subscribe(topic);
        } else {
            tracing::error!("No such client `{}`", client_id);
        }
    }

    /// Unsubscribe a client from an event topic and unsubscribe self if
    /// no more clients are subscribed to that topic.
    fn unsubscribe_topic(
        &self,
        client: &mut WebsocketClient,
        topic: &str,
        subscriptions: &mut HashMap<String, (SubscriptionId, usize)>,
    ) {
        client.unsubscribe(topic);

        if let Entry::Occupied(mut occupied) = subscriptions.entry(topic.to_string()) {
            let (subscription_id, clients) = occupied.get_mut();
            if *clients == 1 {
                if let Err(err) = unsubscribe(subscription_id) {
                    tracing::debug!(
                        "While unsubscribing from subscription `{}`: {}",
                        subscription_id,
                        err,
                    )
                }
                occupied.remove();
            } else {
                *clients -= 1;
            }
        }
    }

    /// Unsubscribe a client from an event topic
    pub async fn unsubscribe(&self, client_id: &str, topic: &str) {
        let mut clients = self.inner.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            let subscriptions = &mut *self.subscriptions.write().await;
            tracing::trace!(
                "Unsubscribing client `{}` from topic `{}`",
                client_id,
                topic
            );
            self.unsubscribe_topic(client, topic, subscriptions);
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

        if let Some(client) = clients.get_mut(client_id) {
            let subscriptions = &mut *self.subscriptions.write().await;
            for topic in client.subscriptions.clone() {
                self.unsubscribe_topic(client, &topic, subscriptions);
            }
        }

        clients.remove(client_id);
    }

    /// Remove all clients from the store
    ///
    /// Removes all clients and all event subscriptions.
    /// This should be done when the server is stopped to avoid keeping a record
    /// of clients that have been disconnected.
    pub async fn clear(&self) {
        let mut subscriptions = self.subscriptions.write().await;
        for (subscription_id, ..) in subscriptions.values() {
            if let Err(err) = unsubscribe(subscription_id) {
                tracing::debug!(
                    "While unsubscribing from subscription `{}`: {}",
                    subscription_id,
                    err,
                )
            }
        }
        subscriptions.clear();

        let mut clients = self.inner.write().await;
        clients.clear();
    }

    /// Ping all clients periodically
    fn ping(clients: Arc<RwLock<HashMap<String, WebsocketClient>>>) {
        tokio::spawn(async move {
            loop {
                let clients = clients.read().await;
                for (client_id, client) in clients.iter() {
                    if let Err(error) = client.sender.send(warp::ws::Message::ping("ping")) {
                        tracing::debug!("While sending ping to client `{}`: {}", client_id, error)
                    }
                }
                // Explicitly drop the read lock so that it is not held while sleeping
                drop(clients);

                use tokio::time::{sleep, Duration};
                sleep(Duration::from_secs(15)).await;
            }
        });
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
        clients: Arc<RwLock<HashMap<String, WebsocketClient>>>,
        receiver: mpsc::UnboundedReceiver<events::Message>,
    ) {
        let mut receiver = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver);
        while let Some((topic, event)) = receiver.next().await {
            tracing::trace!("Received event for topic `{}`", topic);

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

            tracing::trace!(
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
    }
}

/// Return an error response result
///
/// Used to have a consistent structure to error responses in the
/// handler functions below.
fn error_response(code: StatusCode, message: &str) -> warp::reply::Response {
    warp::reply::with_status(
        warp::reply::json(&serde_json::json!({ "message": message })),
        code,
    )
    .into_response()
}

/// Return an error response result
fn error_result(
    code: StatusCode,
    message: &str,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    Ok(error_response(code, message))
}

/// Handle a HTTP `GET` request to the `/~static/` path
#[tracing::instrument]
async fn get_static(
    path: warp::path::Tail,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str().to_string();
    record_http_request("GET", &["/~static/", &path].concat());

    match get_static_parts(&path) {
        Ok((status, header_map, body)) => {
            let mut response = warp::reply::Response::new(body.into());
            *response.status_mut() = status;
            for (name, value) in header_map {
                if let Some(name) = name {
                    response.headers_mut().insert(name, value);
                };
            }
            Ok(response)
        }
        Err(error) => Ok(error_response(error.status, &error.message)),
    }
}

#[tracing::instrument]
async fn get_metrics() -> Result<impl warp::Reply, std::convert::Infallible> {
    record_http_request("GET", "/~metrics");

    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    // Gather custom metrics
    let mut buffer = Vec::new();
    if let Err(error) = encoder.encode(&METRICS_REGISTRY.gather(), &mut buffer) {
        tracing::error!("Could not encode custom metrics: {}", error);
    };
    let mut response = match String::from_utf8(buffer.clone()) {
        Ok(string) => string,
        Err(error) => {
            tracing::error!("Custom metrics could not be stringified: {}", error);
            String::default()
        }
    };

    // Gather default process metrics
    // https://prometheus.io/docs/instrumenting/writing_clientlibs/#process-metrics
    let mut buffer = Vec::new();
    if let Err(error) = encoder.encode(&prometheus::gather(), &mut buffer) {
        tracing::error!("Could not encode Prometheus metrics: {}", error);
    };
    let defaults = match String::from_utf8(buffer.clone()) {
        Ok(string) => string,
        Err(error) => {
            tracing::error!("Prometheus metrics could not be stringified: {}", error);
            String::default()
        }
    };
    response.push_str(&defaults);

    Ok(response)
}

/// Query parameters for `post_hooks`
#[derive(Debug, Deserialize)]
#[serde(crate = "common::serde")]
struct HooksParams {
    src: String,
    dest: Option<PathBuf>,
    mode: Option<providers::WatchMode>,
    token: Option<String>,
}

// Handle a webhook event
//
// Marshals the request into a `server_utils::Request` which is then forwarded
// on to the `providers` internal crate for dispatching based on the `source` query
// parameter.
#[tracing::instrument]
async fn post_hooks(
    params: HooksParams,
    headers: warp::http::HeaderMap,
    json: serde_json::Value,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    record_http_request("POST", "/~hooks");

    let mut builder = http::Request::builder();
    for (name, value) in headers {
        if let Some(name) = name {
            builder = builder.header(name, value);
        }
    }
    let request = builder.body(json).expect("Unable to create request");

    let response = match providers::sync(
        &Node::String(params.src),
        &params.dest.unwrap_or_else(|| PathBuf::from(".")),
        &request,
        Some(providers::SyncOptions {
            mode: params.mode,
            token: params.token,
        }),
    )
    .await
    {
        Ok(response) => warp::reply::with_status(
            warp::reply::Response::new(response.body().to_string().into()),
            StatusCode::OK,
        ),
        Err(error) => warp::reply::with_status(
            warp::reply::Response::new(error.to_string().into()),
            StatusCode::BAD_REQUEST,
        ),
    }
    .into_response();

    Ok(response)
}

/// Query parameters for `authentication_filter`
#[derive(Deserialize)]
#[serde(crate = "common::serde")]
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
            |(key, _home, param, header, cookie): (
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
                        tracing::trace!("Authenticated using param");
                        (Some(param.clone()), jwt::decode(&param, &key))
                    } else {
                        (None, Err(JwtError::NoTokenSupplied))
                    };

                    // Attempt to get from authorization header
                    let (token, claims) = if let (Err(..), Some(header)) = (&claims, header) {
                        tracing::trace!("Authenticated using header");
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
                            tracing::trace!("Authenticated using cookie");
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

                    let project = claims.prn.clone().map(PathBuf::from);

                    // Generate a new token if necessary (single-use or soon to expire) for use in WebSocket URLs
                    // and/or cookies.
                    let updated_token = if claims.jti.is_some()
                        || Utc.timestamp(claims.exp, 0) < Utc::now() + Duration::seconds(60)
                    {
                        jwt::encode(&key, project, Some(YEAR_SECONDS), false)
                            .expect("Should encode")
                    } else {
                        token.clone().unwrap_or_default()
                    };

                    // Provide a token cookie if the claims did not come from a cookie or if it
                    // has been refreshed
                    // Token expires at the end of the browser session and should only be sent to
                    // URL paths that are within the project.
                    let cookie = if !from_cookie || updated_token != token.unwrap_or_default() {
                        Some(format!(
                            "token={}; Path=/; SameSite=Lax; HttpOnly",
                            updated_token
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

/// Handle a request for a HTTP upgrade to the
#[tracing::instrument(skip(_cookie))]
async fn terminal_handler(
    (secure, token, claims, _cookie): (bool, String, jwt::Claims, Option<String>),
) -> Result<warp::reply::Response, std::convert::Infallible> {
    record_http_request("GET", "/~terminal");
    record_activity();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link href="{static_root}/web/terminal.css" rel="stylesheet">
        <script src="{static_root}/web/terminal.js"></script>
    </head>
    <body>
      <div id="terminal-container">
        <div id="terminal"></div>
      </div>
      <script>
        stencilaWebTerminal.main("terminal")
      </script>
    </body>
</html>"#,
        static_root = ["/~static/", STATIC_VERSION].concat()
    );

    let response = warp::reply::Response::new(html.into());
    Ok(response)
}

/// Handle a HTTP request for ~attach
#[tracing::instrument(skip(_cookie))]
fn attach_handler(
    ws: warp::ws::Ws,
    (secure, token, claims, _cookie): (bool, String, jwt::Claims, Option<String>),
) -> Box<dyn warp::Reply> {
    record_http_request("WS", "/~attach");
    record_activity();

    Box::new(ws.on_upgrade(|socket| attach_connected(socket, claims)))
}

/// Handle a WebSocket connection for ~attach
///
/// Pipes data between the websocket connection and the PTY.
async fn attach_connected(web_socket: warp::ws::WebSocket, claims: jwt::Claims) {
    #[allow(unused_mut, unused_variables)]
    let (mut ws_sender, mut ws_receiver) = web_socket.split();
    let (message_sender, mut message_receiver) = mpsc::channel(1);

    #[cfg(target_os = "linux")]
    let child_task = tokio::spawn(async move {
        use pty_process::Command;
        use tokio::io::AsyncReadExt;
        use tokio::io::AsyncWriteExt;

        const CMD: &str = "/bin/bash";
        let mut command = tokio::process::Command::new(CMD);

        // Options necessary to ensure the custom PS1, and other settings are not overridden
        // by profile and init scripts. See https://unix.stackexchange.com/a/291913
        command.args(&["--noprofile", "--norc"]);

        let user = claims.usn.as_deref().unwrap_or("\\u");
        let host = "\\h";
        let dir = "\\w";
        const GREEN: &str = "\\e[1;32m";
        const BLUE: &str = "\\e[0;34m";
        const RESET: &str = "\\e[0m";
        let prompt = format!(
            r"{}{}{}@{}{}{}:{}{}{}$ ",
            GREEN, user, RESET, BLUE, host, RESET, GREEN, dir, RESET
        );
        command.env("PS1", prompt);

        let mut child = match command.spawn_pty(Some(&pty_process::Size::new(50, 80))) {
            Ok(child) => child,
            Err(error) => {
                let message = format!("Unable to start command `{}`: {}", CMD, error);
                message_sender
                    .send(warp::ws::Message::text(message))
                    .await
                    .ok();
                return;
            }
        };

        // Only Ubuntu Linux at least, the bytes read from the PTY do not seem to exceed 4.1k
        let mut buffer = [0; 4096];
        loop {
            tokio::select! {
                message = ws_receiver.next() => {
                    if let Some(Ok(message)) = message {
                        if message.is_ping() {
                            if let Err(error) = message_sender.send(warp::ws::Message::pong("pong")).await {
                                let error = error.to_string();
                                if !error.contains("channel closed") {
                                    tracing::error!("While sending pong message to channel: {}", error)
                                }
                                break;
                            }
                        } else if message.is_pong() {
                            // Ignore
                        } else {
                            record_activity();
                            let buffer = message.as_bytes();
                            if let Err(error) = child.pty_mut().write_all(buffer).await {
                                tracing::error!("While writing message to PTY: {}", error)
                            }
                        }
                    } else if let Some(Err(error)) = message {
                        tracing::error!("While receiving WebSocket message: {}", error);
                        break;
                    } else if message.is_none() {
                        break;
                    }
                },
                bytes_read = child.pty_mut().read(&mut buffer[..]) => {
                    if let Ok(bytes_read) = bytes_read {
                        let message = warp::ws::Message::binary(&buffer[..bytes_read]);
                        if let Err(error) = message_sender.send(message).await {
                            let error = error.to_string();
                            if !error.contains("channel closed") {
                                tracing::error!("While sending binary message to channel: {}", error)
                            }
                            break;
                        }
                    } else if let Err(error) = bytes_read {
                        tracing::error!("While reading bytes from PTY: {}", error);
                        break;
                    }
                }
            };
        }
    });

    #[cfg(not(target_os = "linux"))]
    {
        let message =
            "😢  Web terminal is not currently available on this server operating system.\n";
        message_sender
            .send(warp::ws::Message::text(message))
            .await
            .ok();
    }

    // Receive messages on message channel and forward to WebSocket.
    // Use a timeout so that if there is no other activity we at least send a PING
    // every 15 seconds.
    use tokio::time::{timeout, Duration};
    loop {
        let message = match timeout(Duration::from_secs(15), message_receiver.recv()).await {
            Ok(Some(message)) => message,
            Ok(None) => {
                tracing::trace!("Message channel sender was dropped");
                break;
            }
            Err(..) => warp::ws::Message::ping("ping"),
        };
        if let Err(error) = ws_sender.send(message).await {
            let error = error.to_string();
            if !(error.contains("Connection closed normally") || error.contains("Broken pipe")) {
                tracing::error!(
                    "While sending message to terminal WebSocket client: {}",
                    error
                )
            }
            break;
        }
    }

    // Abort the child process
    #[cfg(target_os = "linux")]
    child_task.abort();
}

/// Query parameters for `get_handler`
#[derive(Debug, Deserialize)]
#[serde(crate = "common::serde")]
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

    record_http_request("GET", path);
    record_activity();

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
        return error_result(
            StatusCode::NOT_FOUND,
            &format!("Requested path `{}` does not exist", fs_path.display()),
        );
    }
    .to_path_buf();

    // Check the path is within the server's `home`
    if !traversal && fs_path.strip_prefix(&home).is_err() {
        return error_result(
            StatusCode::FORBIDDEN,
            "Traversal outside of server's home is not permitted",
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
                return error_result(
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
                let content = match document.dump(Some(format.clone()), None).await {
                    Ok(content) => content,
                    Err(error) => {
                        return error_result(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            &format!("While converting document to {} `{}`", format, error),
                        )
                    }
                };

                let content = match format.as_str() {
                    "html" => html_rewrite(
                        &content,
                        &mode,
                        &theme,
                        &components,
                        &token,
                        &home,
                        &fs_path,
                    ),
                    _ => content,
                }
                .as_bytes()
                .to_vec();

                let mime = mime_guess::from_ext(&format).first_or_octet_stream();

                (content, mime.to_string(), false)
            }
            Err(error) => {
                return error_result(
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
    <link href="{static_root}/web/index.{mode}.css" rel="stylesheet">
    <script src="{static_root}/web/index.{mode}.js"></script>
    <script>
        const startup = stencilaWebClient.main("{client}", "{document}", null, "{token}");
        startup().catch((err) => console.error('Error during startup', err))
    </script>"#,
        static_root = static_root,
        mode = mode,
        client = uuids::generate("cl"),
        token = token,
        document = document.display()
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
#[serde(crate = "common::serde")]
struct WsParams {
    /// The id of the client
    client: Option<String>,
}

/// Perform a WebSocket handshake / upgrade
///
/// This function is called at the start of a WebSocket connection.
/// Each WebSocket connection is authorized to access a single project.
/// Authorization is done by checking the `project` in the JWT claims
/// against the requested path.
#[tracing::instrument(skip(_cookie))]
fn rpc_ws_handler(
    ws: warp::ws::Ws,
    params: WsParams,
    (_secure, _token, _claims, _cookie): (bool, String, jwt::Claims, Option<String>),
) -> Box<dyn warp::Reply> {
    record_http_request("WS", "/~rpc");
    record_activity();

    let client_id = params.client.unwrap_or_else(|| generate("cl").to_string());
    Box::new(ws.on_upgrade(|socket| rpc_ws_connected(socket, client_id)))
}

/// Handle a WebSocket connection
///
/// This function is called after the handshake, when a WebSocket client
/// has successfully connected.
#[tracing::instrument(skip(socket))]
async fn rpc_ws_connected(socket: warp::ws::WebSocket, client_id: String) {
    tracing::trace!("WebSocket client `{}` connected", client_id);

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
                let graceful = if message == "Connection closed normally" {
                    true
                } else {
                    tracing::debug!("Websocket send error `{}`", error);
                    false
                };
                WEBSOCKET_CLIENTS
                    .disconnected(&client_clone, graceful)
                    .await
            }
        }
    });

    // Save / update the client
    WEBSOCKET_CLIENTS.connected(&client_id, client_sender).await;
    WEBSOCKET_CLIENTS_COUNT.inc();

    while let Some(result) = ws_receiver.next().await {
        tracing::trace!("Received WebSocket message from client `{}`", client_id);

        // Get the message
        let message = match result {
            Ok(message) => message,
            Err(error) => {
                let message = error.to_string();
                if message == "WebSocket protocol error: Connection reset without closing handshake"
                {
                    WEBSOCKET_CLIENTS.disconnected(&client_id, false).await
                } else {
                    tracing::error!("WebSocket receive error `{}`", error);
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
                tracing::debug!(
                    "Error when parsing request from client `{}`: {}",
                    client_id,
                    error
                );

                let response = rpc::Response::new(None, None, Some(error));
                WEBSOCKET_CLIENTS.send(&client_id, response).await;
                continue;
            }
        };

        // Record the request
        record_rpc_request(&request.method);
        record_activity();

        // Dispatch the request and send back the response and update subscriptions
        let (response, subscription) = request.dispatch(&client_id).await;
        WEBSOCKET_CLIENTS.send(&client_id, response).await;
        match subscription {
            rpc::Subscription::Subscribe(topic) => {
                WEBSOCKET_CLIENTS.subscribe(&client_id, &topic).await;
            }
            rpc::Subscription::Unsubscribe(topic) => {
                WEBSOCKET_CLIENTS.unsubscribe(&client_id, &topic).await;
            }
            rpc::Subscription::None => (),
        }
    }

    // Record that the client has disconnected gracefully
    WEBSOCKET_CLIENTS.disconnected(&client_id, true).await;
    WEBSOCKET_CLIENTS_COUNT.dec();
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
        Error::invalid_request_error("Invalid HTTP method, path and/or query parameters")
    } else {
        Error::server_error("Unknown error")
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&Response {
            error: Some(error),
            ..Default::default()
        }),
        StatusCode::BAD_REQUEST,
    ))
}

/// Get a hostname to use in an externally accessible URL
///
/// This is used when we need to provide an external service with a URL to
/// connect to a server for this instance e.g. Webhooks.
///
/// If the environment variable `STENCILA_HOSTNAME` is defined then that is used,
/// falling back to the public IP address, falling back to `localhost`.
pub async fn hostname() -> String {
    if let Ok(hostname) = env::var("STENCILA_HOSTNAME") {
        hostname
    } else if let Some(ip) = public_ip::addr().await {
        if ip.is_ipv6() {
            // IP6 addresses need to be surrounded in square brackets to use in a URL
            ["[", &ip.to_string(), "]"].concat()
        } else {
            ip.to_string()
        }
    } else {
        "localhost".into()
    }
}

pub mod config {
    use common::{
        defaults::Defaults,
        serde::{Deserialize, Serialize},
        serde_with::skip_serializing_none,
    };
    use schemars::JsonSchema;
    use validator::Validate;

    /// Server
    ///
    /// Configuration settings for running as a server
    #[skip_serializing_none]
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default, crate = "common::serde")]
    #[schemars(deny_unknown_fields)]
    pub struct ServerConfig {
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

    use cli_utils::{
        clap::{self, Parser},
        result, Result, Run,
    };
    use common::async_trait::async_trait;

    use super::*;

    /// Manage document server
    #[derive(Parser)]
    pub struct Command {
        #[clap(subcommand)]
        pub action: Action,
    }

    #[derive(Parser)]
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
    /// ```sh
    /// $ stencila server start :8000
    /// ```
    ///
    /// To serve on all IPv4 addresses on the machine, instead of only `127.0.0.1`,
    ///
    /// ```sh
    /// $ stencila server start 0.0.0.0
    /// ```
    ///
    /// Or if you prefer, use a complete URL including the scheme e.g.
    ///
    ///```sh
    /// $ stencila server start http://127.0.0.1:9000
    /// ```
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
    #[derive(Parser)]
    #[clap(verbatim_doc_comment)]
    pub struct Start {
        /// The home directory for the server to serve from
        ///
        /// Defaults to the current directory or an ancestor project directory (if the current directory
        /// is within a project).
        home: Option<PathBuf>,

        /// The URL to serve on
        ///
        /// Defaults to the `STENCILA_SERVER_URL` environment variable, the value set in config
        /// or otherwise `http://127.0.0.1:9000`.
        #[clap(short, long, env = "STENCILA_SERVER_URL")]
        url: Option<String>,

        /// Secret key to use for signing and verifying JSON Web Tokens
        ///
        /// Defaults to the `STENCILA_SERVER_KEY` environment variable, the value set in config
        /// or otherwise a randomly generated value.
        #[clap(short, long, env = "STENCILA_SERVER_KEY")]
        key: Option<String>,

        /// Do not require a JSON Web Token to access the server
        ///
        /// For security reasons (any client can access files and execute code) this should be avoided.
        #[clap(long)]
        insecure: bool,

        /// Allow traversal out of the server's home directory
        ///
        /// For security reasons (clients can access any file on the filesystem) this should be avoided.
        #[clap(long)]
        traversal: bool,

        /// Allow root (Linux/Mac OS/Unix) or administrator (Windows) user to serve
        ///
        /// For security reasons (clients may be able to execute code as root) this should be avoided.
        #[clap(long)]
        root: bool,

        /// The maximum number of seconds of inactivity before the server shutsdown
        #[clap(long)]
        max_inactivity: Option<u64>,

        /// The maximum number of seconds that the server should run for
        #[clap(long)]
        max_duration: Option<u64>,

        /// Log each request
        #[clap(long)]
        log_requests: bool,
    }

    #[async_trait]
    impl Run for Start {
        async fn run(&self) -> Result {
            if self.key.is_some() && std::env::var("STENCILA_SERVER_KEY").is_err() {
                tracing::warn!("Server key set on command line could be sniffed by malicious processes; prefer to set it in an environment variable or config file.");
            };

            let join_handle = start(
                self.home.clone(),
                self.url.clone(),
                self.key.clone(),
                self.insecure,
                self.traversal,
                self.root,
                self.max_inactivity,
                self.max_duration,
                self.log_requests,
            )
            .await?;

            // If not in interactive mode then wait for join handle to avoid finishing
            if std::env::var("STENCILA_INTERACT_MODE").is_err() {
                join_handle.await?;
            }

            result::nothing()
        }
    }

    /// Stop the server
    #[derive(Parser)]
    pub struct Stop {}

    #[async_trait]
    impl Run for Stop {
        async fn run(&self) -> Result {
            stop().await?;

            result::nothing()
        }
    }

    /// Show details of the server
    #[derive(Parser)]
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
    #[derive(Parser)]
    pub struct Clients {}

    #[async_trait]
    impl Run for Clients {
        async fn run(&self) -> Result {
            let clients = WEBSOCKET_CLIENTS.inner.read().await;
            result::value(&*clients)
        }
    }
}
