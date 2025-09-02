use std::path::{Path, PathBuf};

use clap::Args;
use eyre::{Result, bail};
use stencila_cli_utils::message;
use tokio::fs::{remove_dir_all, remove_file};

use crate::{MANIFEST_FILENAME, Plugin};

/// Link a local directory as a plugin
#[tracing::instrument]
pub async fn link(target: &Path) -> Result<Plugin> {
    tracing::debug!("Linking plugin directory `{}`", target.display());

    if !target.exists() {
        bail!("Directory `{}` does not exist", target.display())
    }
    let target = target.canonicalize()?;

    // Check that there is a manifest in the directory and that it
    // is valid, erroring if it is not
    let manifest = target.join(MANIFEST_FILENAME);
    if !manifest.exists() {
        bail!(
            "Directory `{}` does not have a `{MANIFEST_FILENAME}` file",
            target.display()
        )
    }
    let plugin = Plugin::read_manifest_from(&manifest)?;

    // If the plugin link or directory already exists then remove it
    let link = Plugin::plugin_dir(&plugin.name, false)?;
    if link.exists() {
        if link.is_file() || link.is_symlink() {
            remove_file(&link).await?
        } else {
            remove_dir_all(&link).await?;
        }
    }

    // Symlink to the directory
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&target, link)?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        symlink_dir(&target, link)?;
    }

    Ok(plugin)
}

/// Link to a local plugin
#[derive(Debug, Default, Args)]
pub struct Link {
    /// The directory to link to
    pub directory: PathBuf,
}

impl Link {
    pub async fn run(self) -> Result<()> {
        let plugin = link(&self.directory).await?;

        message!(
            "🔗 Successfully linked directory `{}` as plugin `{}`",
            self.directory.display(),
            plugin.name
        );

        Ok(())
    }
}
