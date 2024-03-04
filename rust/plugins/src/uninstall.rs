use cli_utils::{message, Message};
use common::{
    clap::{self, Args},
    eyre::Result,
    tokio::fs::remove_dir_all,
    tracing,
};

use crate::Plugin;

/// Uninstall a plugin
#[tracing::instrument]
pub async fn uninstall(name: &str) -> Result<Message> {
    tracing::debug!("Uninstalling plugin `{}`", name);

    let dir = Plugin::plugin_dir(name, false)?;
    let message = if dir.exists() {
        remove_dir_all(dir).await?;
        message!("🗑️ Successfully uninstalled plugin `{}`", name)
    } else {
        message!("Plugin `{}` does not appear to be installed", name)
    };

    Ok(message)
}

/// Uninstall a plugin
#[derive(Debug, Default, Args)]
pub struct UninstallArgs {
    /// The name of the plugin to uninstall
    pub name: String,
}

impl UninstallArgs {
    pub async fn run(self) -> Result<Message> {
        uninstall(&self.name).await
    }
}
