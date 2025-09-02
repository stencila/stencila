use clap::Args;
use eyre::Result;
use stencila_cli_utils::message;

use crate::Plugin;

/// Enable a plugin
#[derive(Debug, Default, Args)]
pub struct Enable {
    /// The name of the plugin to enable
    pub name: String,
}

impl Enable {
    pub async fn run(self) -> Result<()> {
        Plugin::enable(&self.name)?;

        message!("✅ Successfully enabled plugin `{}`", self.name);

        Ok(())
    }
}
