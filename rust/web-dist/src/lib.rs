use std::{
    fs::{create_dir_all, write},
    io::Result,
    path::Path,
};

use rust_embed::RustEmbed;

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
