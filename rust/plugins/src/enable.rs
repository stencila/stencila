use cli_utils::{message, Message};
use common::{
    clap::{self, Args},
    eyre::Result,
    tracing,
};

use crate::Plugin;

/// Enable a plugin
#[tracing::instrument]
pub async fn enable(name: &str) -> Result<Message> {
    tracing::debug!("Enabling plugin `{name}`");

    Plugin::enable(name)?;

    Ok(message!("✅ Successfully enabled plugin `{}`", name))
}

/// Enable a plugin
#[derive(Debug, Default, Args)]
pub struct EnableArgs {
    /// The name of the plugin to enable
    pub name: String,
}

impl EnableArgs {
    pub async fn run(self) -> Result<Message> {
        enable(&self.name).await
    }
}
