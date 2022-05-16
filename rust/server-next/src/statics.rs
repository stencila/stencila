use std::fs;

use axum::{
    extract::Path,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::IntoResponse,
};
use rust_embed::RustEmbed;

use crate::errors::ServerError;

/// Static assets
///
/// During development, these are served from the `static` folder (which
/// has a symlinks to `../web/dist/browser` and other folders.
///
/// At build time these are embedded in the binary.
///
/// Use `include` and `exclude` glob patterns to only include the assets that are required.
#[derive(RustEmbed)]
#[folder = "static"]
#[exclude = "web/*.map"]
struct Statics;

/// The version used in URL paths for static assets
/// Allows for caching control (see [`get_static`]).
pub const STATIC_VERSION: &str = if cfg!(debug_assertions) {
    "dev"
} else {
    env!("CARGO_PKG_VERSION")
};

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
    match get_static_parts(&format!("{}/{}", STATIC_VERSION, path)) {
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
        return Err(ServerError::new(
            StatusCode::UNAUTHORIZED,
            "Path traversal not permitted for static assets",
        ));
    }

    let asset = if cfg!(debug_assertions) {
        // The `rust-embed` crate will load from the filesystem during development but
        // does not allow for symlinks (because, since https://github.com/pyros2097/rust-embed/commit/e1720ce38452c7f94d2ff32d2c120d7d427e2ebe,
        // it checks for path traversal using the canonicalized path). This is problematic for our development workflow which
        // includes live reloading of assets developed in the `web` and `components` modules. Therefore, this
        // re-implements loading of assets from the filesystem.
        let fs_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("static")
            .join(&path);
        match fs::read(&fs_path) {
            Ok(data) => data,
            Err(error) => {
                let error = error.to_string();
                if error.contains("No such file or directory") {
                    return Err(ServerError::new(
                        StatusCode::NOT_FOUND,
                        format!("Filesystem path does not exist: {}", fs_path.display()),
                    ));
                } else {
                    return Err(ServerError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Error reading file `{}`: {}", fs_path.display(), error),
                    ));
                }
            }
        }
    } else {
        match Statics::get(&path) {
            Some(asset) => asset.data.into(),
            None => {
                return Err(ServerError::new(
                    StatusCode::NOT_FOUND,
                    format!("Requested static asset `{}` does not exist", &path),
                ));
            }
        }
    };

    let mut headers = HeaderMap::new();

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    headers.append(
        CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).expect("Unable to create header value"),
    );

    let cache_control = if STATIC_VERSION == "dev" {
        "no-cache"
    } else {
        "max-age=31536000, immutable"
    };
    headers.append(
        CACHE_CONTROL,
        HeaderValue::from_str(cache_control).expect("Unable to create header value"),
    );

    Ok((StatusCode::OK, headers, asset))
}
