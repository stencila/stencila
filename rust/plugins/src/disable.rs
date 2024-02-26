use common::{
    clap::{self, Args},
    eyre::Result,
    tracing,
};

use crate::Plugin;

/// Disable a plugin
#[tracing::instrument]
pub async fn disable(name: &str) -> Result<()> {
    tracing::debug!("Disabling plugin `{name}`");

    Plugin::disable(name)?;

    tracing::info!("☑️ Successfully disabled plugin `{}`", name);

    Ok(())
}

#[derive(Debug, Default, Args)]
pub struct DisableArgs {
    /// The name of the plugin to disable
    pub name: String,
}
