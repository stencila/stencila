use axum::{
    extract::Path,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::IntoResponse,
};

use common::{eyre, tracing};
use statics::Statics;
pub use statics::STATICS_VERSION;

use crate::errors::ServerError;

/// Handle a HTTP `GET /~static/` request
///
/// This path includes the current version number e.g. `/~static/1.2.0/...`.
/// This allows a `Cache-Control` header with long `max-age` and `immutable` (so that browsers do not
/// fetch / parse assets on each request) while also causing the browser cache to be busted for
/// each new version of Stencila.
///
/// During development, the version is set to "dev" and the cache control
/// header is not set (for automatic reloading of re-built assets etc).
pub async fn get_static(Path(path): Path<String>) -> impl IntoResponse {
    get_static_parts(&path)
}

/// Get the raw bytes of a static asset
pub fn get_static_bytes(path: &str) -> Result<Vec<u8>, eyre::Report> {
    match get_static_parts(&format!("{}/{}", STATICS_VERSION, path)) {
        Ok(parts) => Ok(parts.2),
        Err(error) => eyre::bail!("{}", error.message),
    }
}

/// Get a static assets as response parts
pub fn get_static_parts(path: &str) -> Result<(StatusCode, HeaderMap, Vec<u8>), ServerError> {
    // Remove the version number with warnings if it is not present
    // or different to current version
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let path = if parts.len() < 2 {
        tracing::warn!("Expected path to have at least two parts");
        path.to_string()
    } else {
        let version = parts[0];
        if version != STATICS_VERSION {
            tracing::warn!(
                "Requested static assets for a version `{}` not equal to current version `{}`",
                version,
                STATICS_VERSION
            );
        }
        parts[1..].join("/")
    };

    // This is not necessary for production (since the filesystem is not touched) only
    // for development. But to keep dev and prod as consistent as possible it is
    // applied in both contexts.
    if path.contains("..") {
        return Err(ServerError::new(
            StatusCode::UNAUTHORIZED,
            "Path traversal not permitted for static assets",
        ));
    }

    let mut headers = HeaderMap::new();

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    headers.append(
        CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).expect("Unable to create header value"),
    );

    let cache_control = if STATICS_VERSION == "dev" {
        "no-cache"
    } else {
        "max-age=31536000, immutable"
    };
    headers.append(
        CACHE_CONTROL,
        HeaderValue::from_str(cache_control).expect("Unable to create header value"),
    );

    let content = match Statics::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = error.to_string();
            if message.contains("does not exist") {
                return Err(ServerError::new(StatusCode::NOT_FOUND, message));
            } else {
                return Err(ServerError::new(StatusCode::INTERNAL_SERVER_ERROR, message));
            }
        }
    };

    Ok((StatusCode::OK, headers, content))
}
