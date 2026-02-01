use std::{
    fs::{create_dir_all, write},
    io::Result,
    path::Path,
};

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
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../web/dist"]
#[cfg_attr(not(debug_assertions), exclude = "*.map")]
#[exclude = ".gitignore"]
pub struct Web;

impl Web {
    /// Copy the distribution to a file path
    ///
    /// If the `minimal` flag is true then only Brotli and PNG files
    /// are written to the path.
    pub fn to_path(path: &Path, minimal: bool) -> Result<()> {
        for file in Self::iter() {
            if minimal && !(file.ends_with(".br") || file.ends_with(".png")) {
                continue;
            }

            let Some(contents) = Self::get(&file) else {
                continue;
            };

            let dest = path.join(&*file);
            if let Some(parent) = dest.parent() {
                create_dir_all(parent)?;
            }

            write(dest, &contents.data)?
        }

        Ok(())
    }
}
