use rust_embed::RustEmbed;
use stencila_version::STENCILA_VERSION;

/// The path where web assets are served in development mode
const WEB_STATIC_PATH_DEV: &str = "/~static/dev";

/// Get the unversioned path where web assets are served in development mode
pub fn web_static_path_dev() -> String {
    WEB_STATIC_PATH_DEV.to_string()
}

/// Get the versioned path where web assets are served in production mode
pub fn web_static_path_versioned() -> String {
    ["/~static/", STENCILA_VERSION].concat()
}

/// Get the production CDN base URL for web assets
pub fn web_base_cdn() -> String {
    ["https://stencila.dev/web/v", STENCILA_VERSION].concat()
}

/// Get the development CDN base URL for web assets
///
/// Points to the mutable `dev` distribution, published ad-hoc via
/// the `release-web.yml` workflow with `VERSION=dev`.
pub fn web_base_cdn_dev() -> String {
    "https://stencila.dev/web/dev".to_string()
}

/// Web interface distribution bundled into binary
///
/// During development these are served directly from the folder but are
/// embedded into the binary on release builds.
///
/// In release builds, only include Brotli-compressed files and images to
/// minimize binary size (~3 MB vs ~50 MB). In debug builds, also include
/// uncompressed CSS, JS (and source maps for easier debugging) because only
/// uncompressed files are served during dev (see note for `STATIC_ENCODINGS` in
/// rust/server/src/statics.rs).
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../web/dist"]
#[exclude = ".gitignore"]
#[cfg_attr(not(debug_assertions), include = "*.br")]
#[cfg_attr(not(debug_assertions), include = "*.png")]
#[cfg_attr(debug_assertions, include = "*.br")]
#[cfg_attr(debug_assertions, include = "*.png")]
#[cfg_attr(debug_assertions, include = "*.css")]
#[cfg_attr(debug_assertions, include = "*.js")]
#[cfg_attr(debug_assertions, include = "*.map")]
pub struct Web;
