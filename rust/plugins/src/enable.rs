use common::{
    clap::{self, Args},
    eyre::Result,
    tracing,
};

use crate::Plugin;

/// Enable a plugin
#[tracing::instrument]
pub async fn enable(name: &str) -> Result<()> {
    tracing::debug!("Enabling plugin `{name}`");

    Plugin::enable(name)?;

    tracing::info!("✅ Successfully enabled plugin `{}`", name);

    Ok(())
}

#[derive(Debug, Default, Args)]
pub struct EnableArgs {
    /// The name of the plugin to enable
    pub name: String,
}
