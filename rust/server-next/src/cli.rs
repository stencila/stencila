use structopt::StructOpt;

use cli_utils::{async_trait::async_trait, result, Result, Run};

use crate::server::Server;

#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder,
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
pub enum Command {
    Start(Start),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match self {
            Command::Start(action) => action.run().await,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder,
    setting = structopt::clap::AppSettings::ColoredHelp
)]
pub struct Start {}

#[async_trait]
impl Run for Start {
    async fn run(&self) -> Result {
        let mut server = Server::new(None)?;
        let handle = server.start()?;

        // If not in interactive mode then wait for join handle to avoid finishing
        if std::env::var("STENCILA_INTERACT_MODE").is_err() {
            handle.await?;
        }

        result::nothing()
    }
}
