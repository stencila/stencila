use crate::jwt;
use crate::protocols::Protocol;
use crate::rpc::{Error, Request, Response};
use crate::urls;
use anyhow::{bail, Result};
use futures::{FutureExt, StreamExt};
use jwt::JwtError;
use reqwest::StatusCode;
use serde::Deserialize;
use std::env;
use std::str::FromStr;
use warp::{Filter, Reply};

/// Serve JSON-RPC requests at a URL
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
/// ```
/// use stencila::serve::serve;
/// serve(Some("ws://0.0.0.0:1234".to_string()), None);
/// ```
pub async fn serve(url: Option<String>, key: Option<String>) -> Result<()> {
    let url = urls::parse(
        url.unwrap_or_else(|| "ws://127.0.0.1:9000".to_string())
            .as_str(),
    )?;
    let protocol = Protocol::from_str(url.scheme())?;
    let address = url.host().unwrap().to_string();
    let port = url.port_or_known_default();
    serve_on(Some(protocol), Some(address), port, key).await
}

/// Run a server in the current thread.
#[tracing::instrument]
pub fn serve_blocking(url: Option<String>, key: Option<String>) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async { serve(url, key).await })
}

/// Run a server on another thread.
#[tracing::instrument]
pub fn serve_background(url: Option<String>, key: Option<String>) -> Result<()> {
    // Spawn a thread, start a runtime in it, and serve using that runtime.
    // Any errors within the thread are logged because we can't return a
    // `Result` from the thread to the caller of this function.
    std::thread::spawn(move || {
        let _span = tracing::trace_span!("serve_in_background");

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!("{}", error.to_string());
                return;
            }
        };
        match runtime.block_on(async { serve(url, key).await }) {
            Ok(_) => {}
            Err(error) => tracing::error!("{}", error.to_string()),
        };
    });

    Ok(())
}

/// Serve JSON-RPC requests over one of alternative transport protocols
///
/// # Arguments
///
/// - `protocol`: The `Protocol` to serve on (defaults to Websocket)
/// - `address`: The address to listen to (defaults to `127.0.0.1`; only for HTTP and Websocket protocols)
/// - `port`: The port to listen on (defaults to `9000`, only for HTTP and Websocket protocols)
///
/// # Examples
///
/// Listen on http://127.0.0.1:9000,
///
/// ```
/// use stencila::serve::serve_on;
/// use stencila::protocols::Protocol;
/// serve_on(Some(Protocol::Http), Some("127.0.0.1".to_string()), Some(9000), None);
/// ```
///
/// Which is equivalent to,
///
/// ```
/// use stencila::serve::serve_on;
/// use stencila::protocols::Protocol;
/// serve_on(Some(Protocol::Http), None, None, None);
/// ```
///
/// Listen on both http://127.0.0.1:8000 and ws://127.0.0.1:9000,
///
/// ```
/// use stencila::serve::serve_on;
/// use stencila::protocols::Protocol;
/// serve_on(Some(Protocol::Ws), None, None, None);
/// ```
pub async fn serve_on(
    protocol: Option<Protocol>,
    address: Option<String>,
    port: Option<u16>,
    key: Option<String>,
) -> Result<()> {
    let protocol = protocol.unwrap_or(if cfg!(feature = "serve-ws") {
        Protocol::Ws
    } else if cfg!(feature = "serve-http") {
        Protocol::Http
    } else if cfg!(feature = "serve-stdio") {
        Protocol::Stdio
    } else {
        bail!("There are no serve-* features enabled")
    });

    let address: std::net::IpAddr = address.unwrap_or_else(|| "127.0.0.1".to_string()).parse()?;

    let port = port.unwrap_or(9000);

    let key = if key.is_some() {
        let mut key = key.unwrap();
        if key == "insecure" {
            None
        } else {
            key.truncate(64);
            Some(key)
        }
    } else {
        Some(generate_key())
    };

    let _span = tracing::trace_span!(
        "serve",
        %protocol, %address, %port
    );

    tracing::info!(%protocol, %address, %port);
    if let Some(key) = key.clone() {
        tracing::info!("To sign JWTs use key:\n\n  {}", key);
        tracing::info!(
            "To login visit:\n\n  {}\n\nNote: Link valid for one use within 5 minutes.",
            login_url(&key, None)?
        );
    }

    match protocol {
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
                let response = respond(request);
                let json = serde_json::to_string(&response)? + "\n";
                // TODO: unwrap any of these errors and log them
                stdout.write_all(json.as_bytes()).await?;
                stdout.flush().await?
            }
        }
        Protocol::Http | Protocol::Ws => {
            let key_clone = key.clone();
            let login = warp::get()
                .and(warp::path("login"))
                .and(warp::query::<LoginParams>())
                .map(move |params: LoginParams| (key_clone.clone(), params))
                .map(login_handler);

            let authorize = || jwt_filter(key.clone());

            let get = warp::get()
                .and(warp::path::end())
                .and(authorize())
                .map(get_handler);

            let post = warp::post()
                .and(warp::path::end())
                .and(warp::body::json::<Request>())
                .and(authorize())
                .map(post_handler);

            let post_wrap = warp::post()
                .and(warp::path::param())
                .and(warp::body::json::<serde_json::Value>())
                .and(authorize())
                .map(post_wrap_handler);

            let ws = warp::path::end().and(warp::ws()).map(ws_handler);

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
                .or(get)
                .or(post)
                .or(post_wrap)
                .or(ws)
                .with(warp::reply::with::default_header(
                    "server",
                    format!(
                        "Stencila/{} ({})",
                        env!("CARGO_PKG_VERSION"),
                        env::consts::OS
                    ),
                ))
                .with(cors)
                .recover(rejection_handler);

            // Use `try_bind_ephemeral` here to avoid potential panic when using `run`
            let (_address, future) = warp::serve(routes).try_bind_ephemeral((address, port))?;
            future.await
        }
    };

    Ok(())
}

/// Generate a secret key for signing and verifying JSON Web Tokens.
///
/// Returns a secret comprised of 64 URL and command line compatible characters
/// (e.g. so that it can easily be entered on the CLI for the `--key` option of the `request` command).
///
/// Uses 64 bytes because this is the maximum size possible for JWT signing keys.
/// Using a large key for JWT signing reduces the probability of brute force attacks.
/// See https://auth0.com/blog/brute-forcing-hs256-is-possible-the-importance-of-using-strong-keys-to-sign-jwts/.
pub fn generate_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate the login URL given a key, and optionally, the path to redirect to
/// on successful login.
pub fn login_url(key: &str, next: Option<String>) -> Result<String> {
    let token = jwt::encode(key.to_string(), Some(300))?;
    let next = next.unwrap_or_else(|| "/".to_string());
    Ok(format!(
        "http://127.0.0.1:9000/login?token={}&next={}",
        token, next
    ))
}

/// A Warp filter that extracts any JSON Web Token from either the `Authorization` header
/// or the `token` query parameter.
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
                    Ok(jwt::Claims { exp: 0 })
                }
            },
        )
}

#[derive(Debug, Deserialize)]
struct TokenParam {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct LoginParams {
    pub token: Option<String>,
    pub next: Option<String>,
}

/// Handle a HTTP `GET /login` request
///
/// This view is intended for humans so it returns HTML responses telling the
/// human if something failed with the login and what to do about it. Otherwise,
/// it just sets a cookie and redirects them to the next page.
#[allow(clippy::unnecessary_unwrap)]
fn login_handler(key_and_params: (Option<String>, LoginParams)) -> warp::reply::Response {
    let (key, params) = key_and_params;
    let token = params.token;
    let next = params.next.unwrap_or_else(|| "/".to_string());

    fn redirect(next: String) -> warp::reply::Response {
        warp::reply::with_header(
            warp::http::StatusCode::MOVED_PERMANENTLY,
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
        warp::reply::with_status(
            warp::reply::html("No token"),
            warp::http::StatusCode::UNAUTHORIZED,
        )
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
            let cookie =
                warp::http::HeaderValue::from_str(format!("token={}", cookie_token).as_str())
                    .unwrap();
            let headers = response.headers_mut();
            headers.insert("set-cookie", cookie);
            response
        } else {
            // Invalid token
            warp::reply::with_status(
                warp::reply::html("Invalid token"),
                warp::http::StatusCode::UNAUTHORIZED,
            )
            .into_response()
        }
    }
}

/// Handle a HTTP `GET /` request
fn get_handler(_claims: jwt::Claims) -> impl warp::Reply {
    warp::reply::html("👋")
}

/// Handle a HTTP `POST /` request
fn post_handler(request: Request, _claims: jwt::Claims) -> impl warp::Reply {
    let response = respond(request);
    warp::reply::json(&response)
}

/// Handle a HTTP `POST /<method>` request
fn post_wrap_handler(
    method: String,
    params: serde_json::Value,
    _claims: jwt::Claims,
) -> impl warp::Reply {
    use warp::reply;

    // Wrap the method and parameters into a request
    let request = serde_json::from_value::<Request>(serde_json::json!({
        "method": method,
        "params": params
    }));
    let request = match request {
        Ok(request) => request,
        Err(error) => {
            return reply::with_status(
                reply::json(&serde_json::json!({
                    "message": error.to_string()
                })),
                StatusCode::BAD_REQUEST,
            )
        }
    };

    // Unwrap the response into results or error message
    let Response { result, error, .. } = respond(request);
    match result {
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
    }
}

/// Handle a Websocket connection
fn ws_handler(ws: warp::ws::Ws) -> impl warp::Reply {
    ws.on_upgrade(|socket| {
        // TODO Currently just echos
        let (tx, rx) = socket.split();
        rx.forward(tx).map(|result| {
            if let Err(e) = result {
                eprintln!("websocket error: {:?}", e);
            }
        })
    })
}

/// Handle a rejection by converting into a JSON-RPC response
///
/// The above handlers can not handle all errors, in particular, they do not
/// handle JSON parsing errors (which are rejected by the `warp::body::json` filter).
/// This therefore ensures that any request expecting a JSON-RPC response, will get
/// a JSON-RPC response (in these cases containing and error code and message).
async fn rejection_handler(
    rejection: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let code: i16;
    let message: String;

    if let Some(error) = rejection.find::<jwt::JwtError>() {
        code = -1;
        message = format!("{}", error);
    } else if let Some(error) = rejection.find::<warp::filters::body::BodyDeserializeError>() {
        code = -32700;
        message = format!("{}", error);
    } else if rejection.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = -32700;
        message = "Method not found".to_string();
    } else {
        code = -32000;
        message = "Internal server error".to_string();
    }

    Ok(warp::reply::json(&Response {
        error: Some(Error { code, message }),
        ..Default::default()
    }))
}

/// Respond to a request
///
/// Optionally pass a dispatching closure which dispatches the requested method
/// and parameters to a function that returns a result.
fn respond(request: Request) -> Response {
    let id = request.id();
    match request.dispatch() {
        Ok(node) => Response::new(id, Some(node), None),
        Err(error) => Response::new(id, None, Some(error)),
    }
}

#[cfg(feature = "config")]
pub mod config {
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[derive(Debug, PartialEq, Deserialize, Serialize, Validate)]
    pub struct Config {
        /// The URL to serve on (defaults to `ws://127.0.0.1:9000`)
        #[serde(default = "default_url")]
        #[validate(url(message = "Not a valid URL"))]
        pub url: String,

        /// Secret key to use for signing and verifying JSON Web Tokens
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub key: Option<String>,

        /// Do not require a JSON Web Token
        #[serde(default)]
        pub insecure: bool,
    }

    /// Default configuration
    ///
    /// These values are used when `config.toml` does not
    /// contain any config for `serve`.
    impl Default for Config {
        fn default() -> Self {
            Config {
                url: default_url(),
                key: None,
                insecure: false,
            }
        }
    }

    /// Get the default value for `url`
    pub fn default_url() -> String {
        "ws://127.0.0.1:9000".to_string()
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Serve an executor using HTTP, WebSockets, or Standard I/O")]
    pub struct Args {
        /// The URL to serve on (defaults to `ws://127.0.0.1:9000`)
        #[structopt(env = "STENCILA_URL")]
        url: Option<String>,

        /// Secret key to use for signing and verifying JSON Web Tokens
        #[structopt(short, long, env = "STENCILA_KEY")]
        key: Option<String>,

        /// Do not require a JSON Web Token
        #[structopt(long)]
        insecure: bool,
    }

    pub async fn serve(args: Args) -> Result<()> {
        let Args { url, key, insecure } = args;

        let config = crate::config::get()?.serve;
        let url = url.or(Some(config.url));
        let key = key.or(config.key);
        let insecure = insecure || config.insecure;

        super::serve(
            url,
            if insecure {
                Some("insecure".to_string())
            } else {
                key
            },
        )
        .await?;

        Ok(())
    }
}
