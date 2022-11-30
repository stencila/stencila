use std::path::{Path, PathBuf};

use fs_utils::copy_dir_all;
use rust_embed::RustEmbed;

use common::{
    eyre::{bail, Result},
    futures::future,
    tokio::fs::{remove_dir_all, write},
};

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
pub struct Statics;

/// The version used in URL paths for static assets
pub const STATICS_VERSION: &str = if cfg!(debug_assertions) {
    "dev"
} else {
    env!("CARGO_PKG_VERSION")
};

impl Statics {
    /// Get the absolute path of the source statics directory
    fn dir() -> PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("static")
    }

    /// Read an embedded file
    pub fn read(path: &str) -> Result<Vec<u8>> {
        if cfg!(debug_assertions) {
            // The `rust-embed` crate will load from the filesystem during development but
            // does not allow for symlinks (because, since https://github.com/pyros2097/rust-embed/commit/e1720ce38452c7f94d2ff32d2c120d7d427e2ebe,
            // it checks for path traversal using the canonicalized path). This is problematic for our development workflow which
            // includes live reloading of assets developed in the `web` and `components` modules. Therefore, this
            // re-implements loading of assets from the filesystem.
            let path = Statics::dir().join(path);

            // Resist the temptation to make this read async as it is only used during development
            // and doing so would "infect" callers with awaits.
            match std::fs::read(&path) {
                Ok(data) => Ok(data),
                Err(error) => {
                    let error = error.to_string();
                    if error.contains("No such file or directory") {
                        bail!("Filesystem path does not exist: {}", path.display())
                    } else {
                        bail!("Error reading file `{}`: {}", path.display(), error)
                    }
                }
            }
        } else {
            match Statics::get(path) {
                Some(asset) => Ok(asset.data.into()),
                None => bail!("Requested static asset `{}` does not exist", &path),
            }
        }
    }

    /// Write a theme sub-directory of the embedded files to a directory
    ///
    /// TODO: At present, due to how the themes are bundled, the theme name is ignored and
    /// all themes are written
    pub async fn write_theme<P: AsRef<Path>>(dest: P, _name: &str, versioned: bool) -> Result<()> {
        Self::write_sub(dest, "themes", versioned).await
    }

    /// Write the `web` sub-directory of the embedded files to a directory
    ///
    /// TODO: At present, `mode` is ignored but in future, only the static assets for the mode
    /// should be written
    pub async fn write_web<P: AsRef<Path>>(dest: P, _mode: &str, versioned: bool) -> Result<()> {
        Self::write_sub(dest, "web", versioned).await
    }

    /// Write the `components` sub-directory of the embedded files to a directory
    pub async fn write_components<P: AsRef<Path>>(dest: P, versioned: bool) -> Result<()> {
        Self::write_sub(dest, "components", versioned).await
    }

    /// Write a sub-directory of the embedded files to a directory
    ///
    /// When in debug mode we need to copy the statics directory because, as mentioned above,
    /// `rust_embed` does not include symlinks
    async fn write_sub<D, S>(dest: D, sub: S, versioned: bool) -> Result<()>
    where
        D: AsRef<Path>,
        S: AsRef<Path>,
    {
        let dest = dest.as_ref();
        let sub = sub.as_ref();

        let dest = match versioned {
            true => dest.join(STATICS_VERSION).join(sub),
            false => dest.to_path_buf().join(sub),
        };

        if cfg!(debug_assertions) {
            copy_dir_all(Statics::dir().join(sub), dest)?;
        } else {
            let sub = sub.to_string_lossy();
            let sub = sub.as_ref();
            let dest = &dest;
            let futures = Statics::iter().filter_map(|path| {
                path.as_ref().strip_prefix(sub).map(|path| {
                    let path = path.to_string();
                    async move {
                        if let Some(content) = Statics::get(&path) {
                            let file = dest.join(path.clone());
                            let bytes: &[u8] = content.data.as_ref();
                            write(file, bytes).await
                        } else {
                            Ok(())
                        }
                    }
                })
            });
            future::try_join_all(futures).await?;
        }

        Ok(())
    }

    /// Write all embedded files to a directory
    pub async fn write_all<P: AsRef<Path>>(dest: P, versioned: bool) -> Result<()> {
        let dest = match versioned {
            true => dest.as_ref().join(STATICS_VERSION),
            false => dest.as_ref().to_path_buf(),
        };

        if cfg!(debug_assertions) {
            copy_dir_all(Statics::dir(), dest)?;
        } else {
            let dest = &dest;
            let futures = Statics::iter().map(|path| async move {
                if let Some(content) = Statics::get(path.as_ref()) {
                    let file = dest.join(path.as_ref());
                    let bytes: &[u8] = content.data.as_ref();
                    write(file, bytes).await
                } else {
                    Ok(())
                }
            });
            future::try_join_all(futures).await?;
        }

        Ok(())
    }

    /// Remove a statics directory previous written
    pub async fn clean<P: AsRef<Path>>(dir: P) -> Result<()> {
        let dir = dir.as_ref();

        if dir.exists() {
            remove_dir_all(dir).await?;
        }

        Ok(())
    }
}
