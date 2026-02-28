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
    ["https://stencila.io/web/v", STENCILA_VERSION].concat()
}

/// Get the localhost base URL for web assets (for preview/development)
pub fn web_base_localhost(port: u16) -> String {
    ["http://localhost:", &port.to_string(), WEB_STATIC_PATH_DEV].concat()
}

/// Get the default localhost base URL (port 9000)
pub fn web_base_localhost_default() -> String {
    web_base_localhost(9000)
}

/// Web interface distribution bundled into binary
///
/// During development these are served directly from the folder
/// but are embedded into the binary on release builds.
///
/// In release builds, only include Brotli-compressed files and images
/// to minimize binary size (~3 MB vs ~50 MB). In debug builds, also
/// include uncompressed JS and source maps for easier debugging.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../web/dist"]
#[exclude = ".gitignore"]
#[cfg_attr(not(debug_assertions), include = "*.br")]
#[cfg_attr(not(debug_assertions), include = "*.png")]
#[cfg_attr(debug_assertions, include = "*.br")]
#[cfg_attr(debug_assertions, include = "*.png")]
#[cfg_attr(debug_assertions, include = "*.js")]
#[cfg_attr(debug_assertions, include = "*.map")]
pub struct Web;
