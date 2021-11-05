use crate::{
    config::CONFIG,
    documents::DOCUMENTS,
    jwt,
    pubsub::{self, subscribe, Subscriber},
    rpc::{self, Error, Protocol, Request, Response},
    utils::{
        keys, urls,
        uuids::{self, Family},
    },
};
use defaults::Defaults;
use eyre::{bail, Result};
use futures::{SinkExt, StreamExt};
use itertools::Itertools;
use jwt::JwtError;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    env,
    fmt::Debug,
    path::Path,
    str::FromStr,
    sync::Arc,
};
use thiserror::private::PathAsDisplay;
use tokio::sync::{mpsc, RwLock};
use warp::{
    http::{header::HeaderValue, StatusCode},
    ws, Filter, Reply,
};

/// Parse a URL into protocol, address and port components
pub fn parse_url(url: &str) -> Result<(Protocol, String, u16)> {
    let url = urls::parse(url)?;
    let protocol = Protocol::from_str(url.scheme())?;
    let address = url.host().unwrap().to_string();
    let port = url
        .port_or_known_default()
        .expect("Should be a default port for the protocol");
    Ok((protocol, address, port))
}

/// Generate the login URL given a key, and optionally, the path to redirect to
/// on successful login.
pub fn login_url(
    port: u16,
    key: Option<String>,
    expiry_seconds: Option<i64>,
    next: Option<String>,
) -> Result<String> {
    let next = next.unwrap_or_else(|| "/".to_string());
    let url = if let Some(key) = key {
        let token = jwt::encode(key, expiry_seconds)?;
        format!(
            "http://127.0.0.1:{}/~login?token={}&next={}",
            port, token, next
        )
    } else {
        format!("http://127.0.0.1:{}{}", port, next)
    };
    Ok(url)
}

/// Run a server on this thread
///
/// # Arguments
///
/// - `url`: The URL to listen on
/// - `key`: A secret key for signing and verifying JSON Web Tokens (defaults to random)
///
/// # Examples
///
/// Listen on ws://0.0.0.0:1234,
///
/// ```no_run
/// # #![recursion_limit = "256"]
/// use stencila::serve::serve;
///
/// serve("ws://0.0.0.0:1234", None);
/// ```
pub async fn serve(url: &str, key: Option<String>) -> Result<()> {
    let (protocol, address, port) = parse_url(url)?;
    serve_on(protocol, address, port, key).await
}

/// Run a server on another thread
///
/// # Arguments
///
/// - `url`: The URL to listen on
/// - `key`: A secret key for signing and verifying JSON Web Tokens (defaults to random)
#[tracing::instrument]
pub fn serve_background(url: &str, key: Option<String>) -> Result<()> {
    // Spawn a thread, start a runtime in it, and serve using that runtime.
    // Any errors within the thread are logged because we can't return a
    // `Result` from the thread to the caller of this function.
    let url = url.to_string();
    std::thread::spawn(move || {
        let _span = tracing::trace_span!("serve_in_background");

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!("{}", error.to_string());
                return;
            }
        };
        match runtime.block_on(async { serve(&url, key).await }) {
            Ok(_) => {}
            Err(error) => tracing::error!("{}", error.to_string()),
        };
    });

    Ok(())
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

struct Client {
    /// A list of subscription topics for this client
    subscriptions: HashSet<String>,

    /// The current sender for this client
    ///
    /// This is set / reset each time that the client opens
    /// a websocket connection
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl Client {
    pub fn subscribe(&mut self, topic: &str) -> bool {
        self.subscriptions.insert(topic.to_string())
    }

    pub fn unsubscribe(&mut self, topic: &str) -> bool {
        self.subscriptions.remove(topic)
    }

    // Is a client subscribed to a particular topic, or set of topics?
    pub fn subscribed(&self, topic: &str) -> bool {
        for subscription in &self.subscriptions {
            if subscription == "*" || topic.starts_with(subscription) {
                return true;
            }
        }
        false
    }

    pub fn send(&self, message: impl Serialize) {
        match serde_json::to_string(&message) {
            Ok(json) => self.send_text(&json),
            Err(error) => tracing::error!("Error serializing to JSON `{}`", error),
        }
    }

    pub fn send_text(&self, text: &str) {
        if let Err(error) = self.sender.send(warp::ws::Message::text(text)) {
            tracing::error!("Client send error `{}`", error)
        }
    }
}

/// A store of clients
#[derive(Defaults)]
struct Clients {
    clients: Arc<RwLock<HashMap<String, Client>>>,
}

impl Clients {
    pub fn new() -> Self {
        let clients = Clients::default();

        let (sender, receiver) = mpsc::unbounded_channel::<pubsub::Message>();
        subscribe("*", Subscriber::Sender(sender)).unwrap();
        tokio::spawn(Clients::publish(clients.clients.clone(), receiver));

        clients
    }

    pub async fn connected(&self, id: &str, sender: mpsc::UnboundedSender<ws::Message>) {
        let mut clients = self.clients.write().await;
        match clients.entry(id.to_string()) {
            Entry::Occupied(mut occupied) => {
                tracing::debug!("Re-connection for client `{}`", id);
                let client = occupied.get_mut();
                client.sender = sender;
            }
            Entry::Vacant(vacant) => {
                tracing::debug!("New connection for client `{}`", id);
                vacant.insert(Client {
                    subscriptions: HashSet::new(),
                    sender,
                });
            }
        };
    }

    pub async fn disconnected(&self, id: &str, gracefully: bool) {
        let mut clients = self.clients.write().await;
        clients.remove(id);

        if gracefully {
            tracing::debug!("Graceful disconnection by client `{}`", id)
        } else {
            tracing::warn!("Ungraceful disconnection by client `{}`", id)
        }
    }

    pub async fn send(&self, id: &str, message: impl Serialize) {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(id) {
            client.send(message);
        } else {
            tracing::error!("No such client `{}`", id);
        }
    }

    pub async fn subscribe(&self, id: &str, topic: &str) {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(id) {
            tracing::debug!("Subscribing client `{}` to topic `{}`", id, topic);
            client.subscribe(topic);
        } else {
            tracing::error!("No such client `{}`", id);
        }
    }

    pub async fn unsubscribe(&self, id: &str, topic: &str) {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(id) {
            tracing::debug!("Unsubscribing client `{}` from topic `{}`", id, topic);
            client.unsubscribe(topic);
        } else {
            tracing::error!("No such client `{}`", id);
        }
    }

    /// Publish events to clients
    ///
    /// The receiver will receive _all_ events that are published and relay them on to
    /// clients based in their subscriptions.
    async fn publish(
        clients: Arc<RwLock<HashMap<String, Client>>>,
        receiver: mpsc::UnboundedReceiver<pubsub::Message>,
    ) {
        let mut receiver = tokio_stream::wrappers::UnboundedReceiverStream::new(receiver);
        while let Some((topic, event)) = receiver.next().await {
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

            // Send it!
            for client in clients {
                client.send_text(&json)
            }
        }
    }
}

/// The global clients store
static CLIENTS: Lazy<Clients> = Lazy::new(Clients::new);

/// Run a server
///
/// # Arguments
///
/// - `protocol`: The `Protocol` to serve on (defaults to Websocket)
/// - `address`: The address to listen to (defaults to `127.0.0.1`; only for HTTP and Websocket protocols)
/// - `port`: The port to listen on (defaults to `9000`, only for HTTP and Websocket protocols)
///
/// # Examples
///
/// Listen on both http://127.0.0.1:9000 and ws://127.0.0.1:9000,
///
/// ```no_run
/// # #![recursion_limit = "256"]
/// use stencila::rpc::Protocol;
/// use stencila::serve::serve_on;
///
/// serve_on(Protocol::Ws, "127.0.0.1".to_string(), 9000, None);
/// ```
#[tracing::instrument]
pub async fn serve_on(
    protocol: Protocol,
    address: String,
    port: u16,
    key: Option<String>,
) -> Result<()> {
    if let Some(key) = key.as_ref() {
        if key.len() > 64 {
            bail!("Server key should be 64 bytes or less")
        }
    }

    tracing::info!("Serving on {}://{}:{}", protocol, address, port);

    match protocol {
        Protocol::Http | Protocol::Ws => {
            // Static files (assets embedded in binary for which authorization is not required)

            let statics = warp::get()
                .and(warp::path("~static"))
                .and(warp::path::tail())
                .and_then(get_static);

            // Login endpoint (sets authorization cookie)

            let key_clone = key.clone();
            let login = warp::get()
                .and(warp::path("~login"))
                .map(move || key_clone.clone())
                .and(warp::query::<LoginParams>())
                .map(login_handler);

            // The following HTTP and WS endpoints all require authorization (done by `jwt_filter`)

            let authorize = || jwt_filter(key.clone());

            let local = warp::get()
                .and(warp::path("~local"))
                .and(warp::path::tail())
                .and(authorize())
                .and_then(get_local);

            let ws = warp::path("~ws")
                .and(warp::ws())
                .and(warp::query::<WsParams>())
                .and(authorize())
                .map(ws_handshake);

            let get = warp::get()
                .and(warp::path::full())
                .and(warp::query::<GetParams>())
                .and(authorize())
                .and_then(get_handler);

            let post = warp::post()
                .and(warp::path::end())
                .and(warp::body::json::<Request>())
                .and(authorize())
                .and_then(post_handler);

            let post_wrap = warp::post()
                .and(warp::path::param())
                .and(warp::body::json::<serde_json::Value>())
                .and(authorize())
                .and_then(post_wrap_handler);

            // Custom `server` header
            let server = warp::reply::with::default_header(
                "server",
                format!(
                    "Stencila/{} ({})",
                    env!("CARGO_PKG_VERSION"),
                    env::consts::OS
                ),
            );

            // CORS headers to allow from any origin
            let cors = warp::cors()
                .allow_any_origin()
                .allow_headers(vec![
                    "Content-Type",
                    "Referer", // Note that this is an intentional misspelling!
                    "Origin",
                    "Access-Control-Allow-Origin",
                ])
                .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST])
                .max_age(24 * 60 * 60);

            let routes = login
                .or(statics)
                .or(local)
                .or(ws)
                .or(get)
                .or(post)
                .or(post_wrap)
                .with(server)
                .with(cors)
                .recover(rejection_handler);

            // Use `try_bind_ephemeral` here to avoid potential panic when using `run`
            let address: std::net::IpAddr = address.parse()?;
            let (_address, future) = warp::serve(routes).try_bind_ephemeral((address, port))?;
            future.await
        }
        #[cfg(feature = "serve-stdio")]
        Protocol::Stdio => {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

            let stdin = tokio::io::stdin();
            let mut stdout = tokio::io::stdout();

            let buffer = tokio::io::BufReader::new(stdin);
            let mut lines = buffer.lines();
            // TODO capture next_line errors and log them
            while let Some(line) = lines.next_line().await? {
                // TODO capture any json errors and send
                let request = serde_json::from_str::<Request>(&line)?;
                let (response, ..) = request.dispatch("stdio").await;
                let json = serde_json::to_string(&response)? + "\n";
                // TODO: unwrap any of these errors and log them
                stdout.write_all(json.as_bytes()).await?;
                stdout.flush().await?
            }
        }
        #[allow(unreachable_patterns)]
        _ => {
            bail!("Serving over protocol `{:?}` is not enabled", protocol)
        }
    };

    Ok(())
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

/// Handle a HTTP `GET` request to the `/~static/` path
#[tracing::instrument]
async fn get_static(
    path: warp::path::Tail,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str();
    tracing::debug!("GET ~static /{}", path);

    let asset = match Static::get(path) {
        Some(asset) => asset,
        None => return error_response(StatusCode::NOT_FOUND, "Requested path does not exist"),
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = warp::reply::Response::new(asset.data.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}

/// A Warp filter that extracts any JSON Web Token from either the `Authorization` header
/// or the `token` cookie.
fn jwt_filter(
    key: Option<String>,
) -> impl Filter<Extract = (jwt::Claims,), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and(warp::cookie::optional("token"))
        .map(move |header: Option<String>, cookie: Option<String>| (key.clone(), header, cookie))
        .and_then(
            |args: (Option<String>, Option<String>, Option<String>)| async move {
                if let Some(key) = args.0 {
                    let jwt = if let Some(header) = args.1 {
                        match jwt::from_auth_header(header) {
                            Ok(jwt) => jwt,
                            Err(error) => return Err(warp::reject::custom(error)),
                        }
                    } else if let Some(cookie) = args.2 {
                        cookie
                    } else {
                        return Err(warp::reject::custom(JwtError::NoTokenSupplied));
                    };
                    match jwt::decode(jwt, key) {
                        Ok(claims) => Ok(claims),
                        Err(error) => Err(warp::reject::custom(error)),
                    }
                } else {
                    // No key, so just return an empty claim
                    Ok(jwt::Claims { exp: 0 })
                }
            },
        )
}

#[derive(Debug, Deserialize, Clone)]
struct LoginParams {
    pub token: Option<String>,
    pub next: Option<String>,
}

/// Handle a HTTP `GET /~login` request
///
/// This view is intended for humans so it returns HTML responses telling the
/// human if something failed with the login and what to do about it. Otherwise,
/// it just sets a cookie and redirects them to the next page.
#[allow(clippy::unnecessary_unwrap)]
#[tracing::instrument]
fn login_handler(key: Option<String>, params: LoginParams) -> warp::reply::Response {
    tracing::debug!("GET ~login");

    let token = params.token;
    let next = params.next.unwrap_or_else(|| "/".to_string());

    fn redirect(next: String) -> warp::reply::Response {
        warp::reply::with_header(
            StatusCode::MOVED_PERMANENTLY,
            warp::http::header::LOCATION,
            next.as_str(),
        )
        .into_response()
    }

    if key.is_none() {
        // There is no key so nothing further to do other than redirect to `next`
        redirect(next)
    } else if token.is_none() {
        // There is no `?token=` query parameter
        warp::reply::with_status(warp::reply::html("No token"), StatusCode::UNAUTHORIZED)
            .into_response()
    } else {
        let key = key.unwrap();
        let token = token.unwrap();
        if jwt::decode(token, key.clone()).is_ok() {
            // Valid token, so set a new, long-expiry token cookie and
            // redirect to `next`.
            let mut response = redirect(next);
            const DAY: i64 = 24 * 60 * 60;
            let cookie_token = jwt::encode(key, Some(30 * DAY)).unwrap();
            let cookie = HeaderValue::from_str(format!("token={}", cookie_token).as_str()).unwrap();
            let headers = response.headers_mut();
            headers.insert("set-cookie", cookie);
            response
        } else {
            // Invalid token
            warp::reply::with_status(warp::reply::html("Invalid token"), StatusCode::UNAUTHORIZED)
                .into_response()
        }
    }
}

/// Handle a HTTP `GET` request to a `/~local/` path
#[tracing::instrument]
async fn get_local(
    path: warp::path::Tail,
    _claims: jwt::Claims,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str();
    tracing::debug!("GET ~local /{}", path);

    let cwd = std::env::current_dir().expect("Unable to get current working directory");

    let path = match cwd.join(path).canonicalize() {
        Ok(path) => path,
        Err(_) => return error_response(StatusCode::NOT_FOUND, "Requested path does not exist"),
    };

    if path.strip_prefix(&cwd).is_err() {
        return error_response(
            StatusCode::FORBIDDEN,
            "Requested path is outside of current working directory",
        );
    }

    let content = match std::fs::read(&path) {
        Ok(content) => content,
        Err(error) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("When reading file `{}`", error),
            )
        }
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut response = warp::reply::Response::new(content.into());
    response.headers_mut().insert(
        "content-type",
        warp::http::header::HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(response)
}

#[derive(Debug, Deserialize)]
struct GetParams {
    /// The mode, "view", "exec", or "edit"
    mode: Option<String>,

    /// The format to view or edit
    format: Option<String>,

    /// The theme (when format is `html`)
    theme: Option<String>,
}

/// Handle a HTTP `GET` request for a document
///
/// If the requested path starts with `/static` or is not one of the registered file types,
/// then returns the static asset with the `Content-Type` header set.
/// Otherwise, if the requested `Accept` header includes "text/html", viewer's index.html is
/// returned (which, in the background will request the document as JSON). Otherwise,
/// will attempt to determine the desired format from the `Accept` header and convert the
/// document to that.
#[tracing::instrument]
async fn get_handler(
    path: warp::path::FullPath,
    params: GetParams,
    _claims: jwt::Claims,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str();
    tracing::debug!("GET {}", path);

    let cwd = std::env::current_dir().expect("Unable to get current working directory");

    let path = Path::new(path.strip_prefix('/').unwrap_or(path));
    let path = match cwd.join(path).canonicalize() {
        Ok(path) => path,
        Err(_) => return error_response(StatusCode::NOT_FOUND, "Requested path does not exist"),
    };

    if path.strip_prefix(&cwd).is_err() {
        return error_response(
            StatusCode::FORBIDDEN,
            "Requested path is outside of current working directory",
        );
    }

    let mode = params.mode.unwrap_or_else(|| "view".into());
    let format = params.format.unwrap_or_else(|| "html".into());
    let theme = params.theme.unwrap_or_else(|| "wilmore".into());

    match DOCUMENTS.open(&path, None).await {
        Ok(document) => {
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
                "html" => rewrite_html(&content, &mode, &theme, &cwd, &path),
                _ => content,
            };

            let mime = mime_guess::from_ext(&format).first_or_octet_stream();

            let mut response = warp::reply::Response::new(content.into());
            match format.as_str() {
                "html" | "json" => {
                    response.headers_mut().insert(
                        "content-type",
                        warp::http::header::HeaderValue::from_str(mime.as_ref()).unwrap(),
                    );
                }
                _ => {
                    // Temporary serve other content as plain text to avoid browser download
                    // In the future, this will be replace with a code editing view.
                    response.headers_mut().insert(
                        "content-type",
                        warp::http::header::HeaderValue::from_str("text/plain; charset=utf-8")
                            .unwrap(),
                    );
                }
            }
            Ok(response)
        }
        Err(error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("While opening document `{}`", error),
        ),
    }
}

/// Rewrite HTML to serve local files and wrap with desired theme etc.
///
/// Only local files somewhere withing the current working directory are
/// served.
pub fn rewrite_html(body: &str, mode: &str, theme: &str, cwd: &Path, document: &Path) -> String {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#""file://(.*?)""#).expect("Unable to create regex"));

    let body = REGEX.replace_all(body, |captures: &Captures| {
        let path = captures
            .get(1)
            .expect("To always have first capture")
            .as_str();
        let path = match Path::new(path).canonicalize() {
            Ok(path) => path,
            // Redact the path if it can not be canonicalized
            Err(_) => return r#""""#.to_string(),
        };
        match path.strip_prefix(cwd) {
            Ok(path) => ["\"/~local/", &path.display().to_string(), "\""].concat(),
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

        <link href="/~static/web/{mode}.css" rel="stylesheet">
        <script src="/~static/web/{mode}.js"></script>
        <script>
            const startup = stencilaWebClient.main("{url}", "{client}", "{project}", "{snapshot}", "{document}");
            startup().catch((err) => console.error('Error during startup', err))
        </script>

        <link
            href="https://unpkg.com/@stencila/thema/dist/themes/{theme}/styles.css"
            rel="stylesheet">
        <script
            src="https://unpkg.com/@stencila/components/dist/stencila-components/stencila-components.esm.js"
            type="module">
        </script>
        <script
            src="https://unpkg.com/@stencila/components/dist/stencila-components/stencila-components.js"
            type="text/javascript" nomodule="">
        </script>
    </head>
    <body>
        {body}
    </body>
</html>"#,
        mode = mode,
        // TODO: pass url from outside this function?
        url = "ws://127.0.0.1:9000/~ws",
        client = uuids::generate(Family::Client),
        project = "current",
        snapshot = "current",
        document = document.as_display().to_string(),
        theme = theme,
        body = body
    )
}

/// Handle a HTTP `POST /` request
async fn post_handler(
    request: Request,
    _claims: jwt::Claims,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let (response, ..) = request.dispatch("http").await;
    Ok(warp::reply::json(&response))
}

/// Handle a HTTP `POST /<method>` request
async fn post_wrap_handler(
    method: String,
    params: serde_json::Value,
    _claims: jwt::Claims,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    use warp::reply;

    // Wrap the method and parameters into a request
    let request = serde_json::from_value::<Request>(serde_json::json!({
        "method": method,
        "params": params
    }));
    let request = match request {
        Ok(request) => request,
        Err(error) => {
            return Ok(reply::with_status(
                reply::json(&serde_json::json!({
                    "message": error.to_string()
                })),
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    // Unwrap the response into results or error message
    let (Response { result, error, .. }, ..) = request.dispatch("http").await;
    let reply = match result {
        Some(result) => reply::with_status(reply::json(&result), StatusCode::OK),
        None => match error {
            Some(error) => reply::with_status(reply::json(&error), StatusCode::BAD_REQUEST),
            None => reply::with_status(
                reply::json(&serde_json::json!({
                    "message": "Response had neither a result nor an error"
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        },
    };
    Ok(reply)
}

/// Parameters for the WebSocket handshake
#[derive(Debug, Deserialize)]
struct WsParams {
    client: String,
}

/// Perform a WebSocket handshake / upgrade
///
/// This function is called at the start of a WebSocket connection.
#[tracing::instrument]
fn ws_handshake(ws: warp::ws::Ws, params: WsParams, _claims: jwt::Claims) -> impl warp::Reply {
    tracing::debug!("WebSocket handshake");
    ws.on_upgrade(|socket| ws_connected(socket, params.client))
}

/// Handle a WebSocket connection
///
/// This function is called after the handshake, when a WebSocket client
/// has successfully connected.
#[tracing::instrument(skip(socket))]
async fn ws_connected(socket: warp::ws::WebSocket, client: String) {
    tracing::debug!("WebSocket connected");

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the client's websocket.
    let (client_sender, client_receiver) = mpsc::unbounded_channel();
    let mut client_receiver = tokio_stream::wrappers::UnboundedReceiverStream::new(client_receiver);

    let client_clone = client.clone();
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
    CLIENTS.connected(&client, client_sender).await;

    while let Some(result) = ws_receiver.next().await {
        // Get the message
        let message = match result {
            Ok(message) => message,
            Err(error) => {
                let message = error.to_string();
                if message == "WebSocket protocol error: Connection reset without closing handshake"
                {
                    CLIENTS.disconnected(&client, false).await
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
                CLIENTS.send(&client, response).await;
                continue;
            }
        };

        // Dispatch the request and send back the response and update subscriptions
        let (response, subscription) = request.dispatch(&client).await;
        CLIENTS.send(&client, response).await;
        match subscription {
            rpc::Subscription::Subscribe(topic) => {
                CLIENTS.subscribe(&client, &topic).await;
            }
            rpc::Subscription::Unsubscribe(topic) => {
                CLIENTS.unsubscribe(&client, &topic).await;
            }
            rpc::Subscription::None => (),
        }
    }

    // Record that the client has diconnected gracefully
    CLIENTS.disconnected(&client, true).await
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

    tracing::error!("{:?}", error);

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
    use validator::Validate;

    /// Server
    ///
    /// Configuration settings for running as a server
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default)]
    #[schemars(deny_unknown_fields)]
    pub struct ServeConfig {
        /// The URL to serve on (defaults to `ws://127.0.0.1:9000`)
        #[def = "\"ws://127.0.0.1:9000\".to_string()"]
        #[validate(url(message = "Not a valid URL"))]
        pub url: String,

        /// Secret key to use for signing and verifying JSON Web Tokens
        #[def = "None"]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub key: Option<String>,

        /// Do not require a JSON Web Token to access the server
        #[def = "false"]
        pub insecure: bool,
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use async_trait::async_trait;
    use cli_utils::{result, Result, Run};
    use structopt::StructOpt;

    /// Serve over HTTP and WebSockets
    ///
    /// ## Ports, protocols, and addresses
    ///
    /// Use the <url> argument to change the port, address, and/or protocol that the server
    /// listens on. This argument can be a partial, or complete, URL.
    ///
    /// For example, to serve on port 8000 instead of the default port,
    ///
    ///    stencila serve :8000
    ///
    /// To serve on all IPv4 addresses on the machine, instead of only `127.0.0.1`,
    ///
    ///    stencila serve 0.0.0.0
    ///
    /// To only serve HTTP, and not both HTTP and WebSockets (the default), also specify
    /// the scheme e.g.
    ///
    ///   stencila serve http://127.0.0.1:9000
    ///
    /// ## Security
    ///
    /// By default, the server requires an initial login via a JSON Web Token. A login URL is
    /// printed in the console's standard output at startup. To turn authorization off, for example
    /// if you are using some other security layer in front of the server, use the `--insecure`
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
    pub struct Command {
        /// The URL to serve on (defaults to `ws://127.0.0.1:9000`)
        #[structopt(env = "STENCILA_URL")]
        url: Option<String>,

        /// Secret key to use for signing and verifying JSON Web Tokens
        #[structopt(short, long, env = "STENCILA_KEY")]
        key: Option<String>,

        /// Serve in a background thread (when in interactive mode)
        #[structopt(short, long)]
        background: bool,

        /// Do not require a JSON Web Token to access the server
        #[structopt(long)]
        insecure: bool,

        /// Allow root (Linux/Mac OS/Unix) or administrator (Windows) user to serve
        #[structopt(long)]
        root: bool,
    }
    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let config = &CONFIG.lock().await.serve;

            let url = self.url.clone().unwrap_or_else(|| config.url.clone());
            let (protocol, address, port) = parse_url(&url)?;

            // Get key configured on command line or config file
            let key = match &self.key {
                Some(key) => {
                    tracing::warn!("Server key set on command line can be sniffed by malicious processes; prefer to set it in config file.");
                    Some(key.clone())
                }
                None => config.key.clone(),
            };

            // Check that user is explicitly allowing no key to be used
            let insecure = self.insecure || config.insecure;
            if insecure {
                tracing::warn!("Serving in insecure mode is dangerous and discouraged.")
            }

            // Generate key if necessary
            let key = if key.is_none() {
                match insecure {
                    true => None,
                    false => Some(keys::generate()),
                }
            } else {
                key
            };

            // If stdout is not a TTY then print the login URL to stdout so that it can be used
            // by, for example, the parent process.
            // TODO: Consider re-enabling this when/id `cli` modules are moved to the `cli` crate
            // where the `atty` crate is available. Until then skip to avoid noise on stdout.
            // println!("{}", login_url(port, key.clone(), Some(300), None)?);

            // Check for root usage
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            if let sudo::RunningAs::Root = sudo::check() {
                if self.root {
                    tracing::warn!("Serving as root/administrator is dangerous and discouraged.")
                } else {
                    bail!("Serving as root/administrator is not permitted by default, use the `--root` option to bypass this safety measure.")
                }
            }

            if self.background {
                super::serve_background(&format!("{}://{}:{}", protocol, address, port), key)?;
            } else {
                super::serve_on(protocol, address, port, key).await?;
            }

            result::nothing()
        }
    }
}
