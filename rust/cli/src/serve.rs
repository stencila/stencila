use std::net::{IpAddr, SocketAddr};

use axum::{
    body,
    extract::Path,
    http::{
        header::{ACCEPT_ENCODING, CACHE_CONTROL, CONTENT_ENCODING, CONTENT_TYPE},
        HeaderMap, StatusCode,
    },
    response::{Html, IntoResponse, Response},
    routing::get,
    Router, Server,
};
use rust_embed::RustEmbed;

use common::{eyre, tracing};

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
#[exclude = "*.map"]
struct Static;

/// Start the server
pub async fn serve(address: IpAddr, port: u16) -> eyre::Result<()> {
    let router = Router::new()
        .route("/", get(home))
        .route("/static/*path", get(static_file));

    let address = SocketAddr::new(address, port);

    tracing::info!("Starting server at http://{address}");

    Server::bind(&address)
        .serve(router.into_make_service())
        .await?;

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

/// Get a static file (e.g. `index.js``)
///
/// Paths to static files include a version so that, in production, the cache control
/// header can be set such that clients should only ever need to make a single request
/// for each version of a static file.
///
/// This cache control is turned off in development so that changes to those files
/// propagate to the browser.
#[tracing::instrument]
async fn static_file(Path(path): Path<String>, headers: HeaderMap) -> Response {
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

                if !(cfg!(debug_assertions)) {
                    response = response.header(CACHE_CONTROL, "max-age=31536000, immutable");
                }

                return response
                    .body(body::boxed(body::Full::from(file.data)))
                    .expect("should build");
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
