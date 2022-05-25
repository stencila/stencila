use structopt::StructOpt;

use cli_utils::{async_trait::async_trait, Result, Run};

/// Build and distribute container images
#[derive(Debug, StructOpt)]
#[structopt(
    alias = "images",
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::DeriveDisplayOrder,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
pub enum Command {}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        todo!()
    }
}
