use clap::Args;
use cli_utils::message;
use eyre::Result;

use crate::Plugin;

/// Disable a plugin
#[derive(Debug, Default, Args)]
pub struct Disable {
    /// The name of the plugin to disable
    pub name: String,
}

impl Disable {
    pub async fn run(self) -> Result<()> {
        Plugin::disable(&self.name)?;

        message!("☑️ Successfully disabled plugin `{}`", self.name);

        Ok(())
    }
}
