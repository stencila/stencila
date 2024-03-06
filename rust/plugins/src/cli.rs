use cli_utils::ToStdout;
use common::{
    clap::{self, Parser, Subcommand},
    eyre::Result,
};

use crate::{
    check::CheckArgs, disable::DisableArgs, enable::EnableArgs, install::InstallArgs,
    link::LinkArgs, list::ListArgs, show::ShowArgs, uninstall::UninstallArgs,
};

/// Manage plugins
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List(ListArgs),
    Install(InstallArgs),
    Uninstall(UninstallArgs),
    Link(LinkArgs),
    Enable(EnableArgs),
    Disable(DisableArgs),
    Show(ShowArgs),
    Check(CheckArgs),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            ListArgs::default().run().await?.to_stdout();
            return Ok(());
        };

        match command {
            Command::List(args) => args.run().await?.to_stdout(),
            Command::Install(args) => args.run().await?.to_stdout(),
            Command::Uninstall(args) => args.run().await?.to_stdout(),
            Command::Link(args) => args.run().await?.to_stdout(),
            Command::Enable(args) => args.run().await?.to_stdout(),
            Command::Disable(args) => args.run().await?.to_stdout(),
            Command::Show(args) => args.run().await?.to_stdout(),
            Command::Check(args) => args.run().await?.to_stdout(),
        }

        Ok(())
    }
}
