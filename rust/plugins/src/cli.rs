use cli_utils::ToStdout;
use common::{
    clap::{self, Parser, Subcommand},
    eyre::Result,
};

use crate::list::{list, ListOptions};

/// List, install, update, and uninstall plugins
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// A command to perform with plugins
#[derive(Debug, Subcommand)]
enum Command {
    /// List plugins available and installed
    List(ListOptions),
}

impl Cli {
    // Run the CLI
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            list(ListOptions::default()).await?.to_stdout();
            return Ok(());
        };

        match command {
            Command::List(options) => list(options).await?.to_stdout(),
        }

        Ok(())
    }
}
