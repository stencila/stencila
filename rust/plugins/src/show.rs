use clap::Args;
use cli_utils::ToStdout;
use eyre::Result;

use crate::Plugin;

/// Show details of a plugin
#[derive(Debug, Default, Args)]
pub struct Show {
    /// The name of the plugin to install
    pub name: String,
}

impl Show {
    pub async fn run(self) -> Result<()> {
        let plugin = Plugin::read_manifest(&self.name)?;
        plugin.show().to_stdout();

        Ok(())
    }
}
