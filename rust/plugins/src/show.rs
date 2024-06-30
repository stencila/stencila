use cli_utils::Code;
use common::{
    clap::{self, Args},
    eyre::Result,
    tracing,
};

use crate::Plugin;

/// Show details of a plugin
#[tracing::instrument]
pub async fn show(name: &str) -> Result<Code> {
    tracing::debug!("Showing plugin `{name}`");

    Plugin::read_manifest(name).map(|plugin| plugin.show())
}

/// Show details of a plugin
#[derive(Debug, Default, Args)]
pub struct ShowArgs {
    /// The name of the plugin to install
    pub name: String,
}

impl ShowArgs {
    pub async fn run(self) -> Result<Code> {
        show(&self.name).await
    }
}
