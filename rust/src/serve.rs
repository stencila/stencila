use crate::{
    documents::Documents,
    jwt,
    rpc::{Error, Protocol, Request, Response},
    utils::urls,
};
use eyre::{bail, Result};
use futures::{FutureExt, StreamExt};
use jwt::JwtError;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use reqwest::{header::HeaderValue, StatusCode};
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::str::FromStr;
use std::{env, fmt::Debug, path::Path, sync::Arc};
use tokio::sync::Mutex;
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
/// ```no_run
/// # #![recursion_limit = "256"]
/// use stencila::documents::Documents;
/// use stencila::serve::serve;
///
/// let mut documents = Documents::default();
/// serve(&mut documents, Some("ws://0.0.0.0:1234".to_string()), None);
/// ```
pub async fn serve(
    documents: &mut Documents,
    url: Option<String>,
    key: Option<String>,
) -> Result<()> {
    let url = urls::parse(
        url.unwrap_or_else(|| "ws://127.0.0.1:9000".to_string())
            .as_str(),
    )?;
    let protocol = Protocol::from_str(url.scheme())?;
    let address = url.host().unwrap().to_string();
    let port = url.port_or_known_default();

    let documents = Arc::new(Mutex::new(documents.clone()));
    serve_on(documents, Some(protocol), Some(address), port, key).await
}

/// Run a server on another thread.
#[tracing::instrument]
pub fn serve_background(
    documents: &mut Documents,
    url: Option<String>,
    key: Option<String>,
) -> Result<()> {
    // Spawn a thread, start a runtime in it, and serve using that runtime.
    // Any errors within the thread are logged because we can't return a
    // `Result` from the thread to the caller of this function.
    let mut documents = documents.clone();
    std::thread::spawn(move || {
        let _span = tracing::trace_span!("serve_in_background");

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!("{}", error.to_string());
                return;
            }
        };
        match runtime.block_on(async { serve(&mut documents, url, key).await }) {
            Ok(_) => {}
            Err(error) => tracing::error!("{}", error.to_string()),
        };
    });

    Ok(())
}

/// Static assets
#[cfg(feature = "serve-static")]
#[derive(RustEmbed)]
#[folder = "static"]
struct Static;

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
/// Listen on both http://127.0.0.1:9000 and ws://127.0.0.1:9000,
///
/// ```no_run
/// # #![recursion_limit = "256"]
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
/// use stencila::documents::Documents;
/// use stencila::rpc::Protocol;
/// use stencila::serve::serve_on;
///
/// serve_on(
///     Arc::new(Mutex::new(Documents::default())),
///     Some(Protocol::Ws),
///     Some("127.0.0.1".to_string()),
///     Some(9000),
///     None
/// );
/// ```
pub async fn serve_on(
    documents: Arc<Mutex<Documents>>,
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
        tracing::info!("To sign JWTs use this key: {}", key);
        tracing::info!(
            "To login visit this URL (valid for 5 minutes): {}",
            login_url(&key, Some(300), None)?
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
                let response = respond(request).await;
                let json = serde_json::to_string(&response)? + "\n";
                // TODO: unwrap any of these errors and log them
                stdout.write_all(json.as_bytes()).await?;
                stdout.flush().await?
            }
        }
        Protocol::Http | Protocol::Ws => {
            let statics = warp::get()
                .and(warp::path("~static"))
                .and(warp::path::tail())
                .and_then(get_static);

            let key_clone = key.clone();

            let login = warp::get()
                .and(warp::path("~login"))
                .map(move || key_clone.clone())
                .and(warp::query::<LoginParams>())
                .map(login_handler);

            let authorize = || jwt_filter(key.clone());

            fn with_documents(
                documents: Arc<Mutex<Documents>>,
            ) -> impl Filter<Extract = (Arc<Mutex<Documents>>,), Error = std::convert::Infallible> + Clone
            {
                warp::any().map(move || documents.clone())
            }

            let local = warp::get()
                .and(warp::path("~local"))
                .and(warp::path::tail())
                .and(authorize())
                .and_then(get_local);

            let get = warp::get()
                .and(with_documents(documents))
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
                .or(statics)
                .or(local)
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

/// Return an error response
///
/// Used to have a consistent structure to error responses in the
/// handler functions below.
#[allow(clippy::unnecessary_wraps)]
fn error_response(
    code: warp::http::StatusCode,
    message: &str,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({ "message": message })),
        code,
    )
    .into_response())
}

/// Handle a HTTP `GET` request to the `/~static/` path
async fn get_static(
    path: warp::path::Tail,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str();
    let asset = match Static::get(path) {
        Some(asset) => asset,
        None => return error_response(StatusCode::NOT_FOUND, "Requested path does not exist"),
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = warp::reply::Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}

/// Generate a secret key for signing and verifying JSON Web Tokens.
///
/// Returns a secret comprised of 64 URL and command line compatible characters
/// (e.g. so that it can easily be entered on the CLI for the `--key` option of the `request` command).
///
/// Uses 64 bytes because this is the maximum size possible for JWT signing keys.
/// Using a large key for JWT signing reduces the probability of brute force attacks.
/// See <https://auth0.com/blog/brute-forcing-hs256-is-possible-the-importance-of-using-strong-keys-to-sign-jwts/>.
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
pub fn login_url(key: &str, expiry_seconds: Option<i64>, next: Option<String>) -> Result<String> {
    let token = jwt::encode(key.to_string(), expiry_seconds)?;
    let next = next.unwrap_or_else(|| "/".to_string());
    Ok(format!(
        "http://127.0.0.1:9000/~login?token={}&next={}",
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
fn login_handler(key: Option<String>, params: LoginParams) -> warp::reply::Response {
    let token = params.token;
    let next = params.next.unwrap_or_else(|| "/".to_string());

    tracing::info!("GET login");

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

/// Handle a HTTP `GET` request to a `/~local/` path
async fn get_local(
    path: warp::path::Tail,
    _claims: jwt::Claims,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str();
    tracing::info!("GET (local) /{}", path);

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
                &format!("When reading file: {}", error),
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
    /// The format desired
    format: Option<String>,

    /// The theme desired (for format `html`)
    theme: Option<String>,
}

/// Handle a HTTP `GET` request for a document
///
/// If the requested path starts with `/static` or is not one of the registered file types,
/// then returns the static asset with the
/// `Content-Type` header set.
/// Otherwise, if the requested `Accept` header includes "text/html", viewer's index.html is
/// returned (which, in the background will request the document as JSON). Otherwise,
/// will attempt to determine the desired format from the `Accept` header and convert the
/// document to that.
#[tracing::instrument(skip(documents))]
async fn get_handler(
    documents: Arc<Mutex<Documents>>,
    path: warp::path::FullPath,
    params: GetParams,
    _claims: jwt::Claims,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let path = path.as_str();
    tracing::info!("GET {}", path);

    let cwd = std::env::current_dir().expect("Unable to get current working directory");

    let path = Path::new(path.strip_prefix("/").unwrap_or(path));
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

    let format = params.format.unwrap_or_else(|| "html".into());
    let theme = params.theme.unwrap_or_else(|| "wilmore".into());

    let mut documents = documents.lock().await;
    match documents.open(path, None).await {
        Ok(document) => {
            let content = match documents.dump(&document.id, Some(format.clone())).await {
                Ok(content) => content,
                Err(error) => {
                    return error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("While converting document to {}: {}", format, error),
                    )
                }
            };

            let content = match format.as_str() {
                "html" => rewrite_html(&content, &theme, &cwd),
                _ => content,
            };

            let mime = mime_guess::from_ext(&format).first_or_octet_stream();

            let mut response = warp::reply::Response::new(content.into());
            response.headers_mut().insert(
                "content-type",
                warp::http::header::HeaderValue::from_str(mime.as_ref()).unwrap(),
            );
            Ok(response)
        }
        Err(error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("While opening document: {}", error),
        ),
    }
}

/// Rewrite HTML to serve local files and wrap with desired theme etc.
///
/// Only local files somewhere withing the current working directory are
/// served.
pub fn rewrite_html(body: &str, theme: &str, cwd: &Path) -> String {
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
        <link
            href="https://unpkg.com/@stencila/thema/dist/themes/{theme}/styles.css"
            rel="stylesheet">
        <script
            src="https://unpkg.com/@stencila/components/dist/stencila-components/stencila-components.esm.js"
            type="module"></script>
        <script
            src="https://unpkg.com/@stencila/components/dist/stencila-components/stencila-components.js"
            type="text/javascript" nomodule=""></script>
        <style>
            .todo {{
                font-family: mono;
                color: #f88;
                background: #fff2f2;
            }}
            .unsupported {{
                font-family: mono;
                color: #777;
                background: #eee;
            }}
        </style>
    </head>
    <body>
        <div data-itemscope="root">{body}</div>
    </body>
</html>"#,
        theme = theme,
        body = body
    )
}

/// Handle a HTTP `POST /` request
async fn post_handler(
    request: Request,
    _claims: jwt::Claims,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let response = respond(request).await;
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
    let Response { result, error, .. } = respond(request).await;
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
    let error = if let Some(error) = rejection.find::<jwt::JwtError>() {
        Error::invalid_request_error(&format!("{}", error))
    } else if let Some(error) = rejection.find::<warp::filters::body::BodyDeserializeError>() {
        Error::invalid_request_error(&format!("{}", error))
    } else if rejection.find::<warp::reject::MethodNotAllowed>().is_some() {
        Error::invalid_request_error("Invalid HTTP method and/or path")
    } else {
        Error::server_error("Unknown error")
    };

    Ok(warp::reply::json(&Response {
        error: Some(error),
        ..Default::default()
    }))
}

/// Respond to a request
///
/// Optionally pass a dispatching closure which dispatches the requested method
/// and parameters to a function that returns a result.
async fn respond(request: Request) -> Response {
    let id = request.id();
    match request.dispatch().await {
        Ok(node) => Response::new(id, Some(node), None),
        Err(error) => Response::new(id, None, Some(error)),
    }
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

        /// Do not require a JSON Web Token
        #[def = "false"]
        pub insecure: bool,
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use crate::cli::display;

    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Serve on HTTP, WebSockets, or Standard I/O",
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

        /// Do not require a JSON Web Token
        #[structopt(long)]
        insecure: bool,
    }

    impl Command {
        pub async fn run(self, documents: &mut Documents) -> display::Result {
            let Command { url, key, insecure } = self;

            let config = &crate::config::lock().await.serve;

            let url = url.or_else(|| Some(config.url.clone()));
            let key = key.or_else(|| config.key.clone());
            let insecure = insecure || config.insecure;

            super::serve(
                documents,
                url,
                if insecure {
                    Some("insecure".to_string())
                } else {
                    key
                },
            )
            .await?;

            display::nothing()
        }
    }
}
