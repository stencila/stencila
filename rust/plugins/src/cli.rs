use cli_utils::ToStdout;
use common::{
    clap::{self, Parser, Subcommand},
    eyre::Result,
};

use crate::{
    check::{check, CheckArgs},
    install::{install, InstallArgs},
    list::{list, ListArgs},
    uninstall::{uninstall, UninstallArgs},
};

/// List, install, update, and uninstall plugins
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// A command to perform with plugins
#[derive(Debug, Subcommand)]
enum Command {
    /// List plugins
    List(ListArgs),

    /// Install a plugin
    Install(InstallArgs),

    /// Uninstall a plugin
    Uninstall(UninstallArgs),

    /// Check a plugin
    Check(CheckArgs),
}

impl Cli {
    // Run the CLI
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            list(ListArgs::default()).await?.to_stdout();
            return Ok(());
        };

        match command {
            Command::List(options) => list(options).await?.to_stdout(),
            Command::Install(InstallArgs { name }) => install(&name).await?,
            Command::Uninstall(UninstallArgs { name }) => uninstall(&name).await?,
            Command::Check(CheckArgs { name, transport }) => check(&name, transport).await?,
        }

        Ok(())
    }
}
