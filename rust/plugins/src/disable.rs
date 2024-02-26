use cli_utils::{message, Message};
use common::{
    clap::{self, Args},
    eyre::Result,
    tracing,
};

use crate::Plugin;

/// Disable a plugin
#[tracing::instrument]
pub async fn disable(name: &str) -> Result<Message> {
    tracing::debug!("Disabling plugin `{name}`");

    Plugin::disable(name)?;

    Ok(message!("☑️ Successfully disabled plugin `{}`", name))
}

/// Disable a plugin
#[derive(Debug, Default, Args)]
pub struct DisableArgs {
    /// The name of the plugin to disable
    pub name: String,
}

impl DisableArgs {
    pub async fn run(self) -> Result<Message> {
        disable(&self.name).await
    }
}
