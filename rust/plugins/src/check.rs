use common::{
    clap::{self, Args},
    eyre::Result,
    tracing,
};

use crate::{Plugin, PluginTransport};

/// Install a plugin
#[tracing::instrument]
pub async fn check(name: &str, transport: Option<PluginTransport>) -> Result<()> {
    tracing::debug!("Checking plugin `{name}`");

    let plugin = Plugin::read_manifest(name)?;

    // Start a plugin instance
    let mut instance = plugin.start(transport).await?;

    // Call all methods (with any args) and ensure they don't error
    instance.health().await?;

    // Stop the plugin instance
    instance.stop().await?;

    tracing::info!(
        "Successfully checked plugin `{}` version `{}`",
        plugin.name,
        plugin.version,
    );

    Ok(())
}

#[derive(Debug, Default, Args)]
pub struct CheckArgs {
    /// The name of the plugin to install
    pub name: String,

    /// The message transport to check the plugin with
    pub transport: Option<PluginTransport>,
}
