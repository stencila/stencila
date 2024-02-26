use common::{
    clap::{self, Args},
    eyre::Result,
    tokio::fs::remove_dir_all,
    tracing,
};

use crate::Plugin;

/// Uninstall a plugin
#[tracing::instrument]
pub async fn uninstall(name: &str) -> Result<()> {
    tracing::debug!("Uninstalling plugin `{}`", name);

    let dir = Plugin::plugin_dir(name, false)?;
    if dir.exists() {
        remove_dir_all(dir).await?;
        tracing::info!("🗑️ Successfully uninstalled plugin {}", name);
    } else {
        tracing::warn!("Plugin {} does not appear to be installed", name);
    }

    Ok(())
}

#[derive(Debug, Default, Args)]
pub struct UninstallArgs {
    /// The name of the plugin to uninstall
    pub name: String,
}
