use common::{
    clap::{self, Parser, Subcommand},
    eyre::Result,
};

/// Publish one or more documents
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    publisher: Publisher,
}

#[derive(Debug, Subcommand)]
pub enum Publisher {
    #[command(after_help(publish_zenodo::AFTER_HELP))]
    #[command(after_long_help(publish_zenodo::AFTER_LONG_HELP))]
    Zenodo(Box<publish_zenodo::Cli>),
    Ghost(Box<publish_ghost::Cli>),
    Stencila(Box<publish_stencila::Cli>),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        use Publisher::*;
        match self.publisher {
            Ghost(cli) => cli.run().await,
            Stencila(cli) => cli.run().await,
            Zenodo(cli) => cli.run().await,
        }
    }
}
