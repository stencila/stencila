use clap::{Parser, Subcommand};
use eyre::Result;

/// Publish one or more documents
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    publisher: Publisher,
}

#[derive(Debug, Subcommand)]
pub enum Publisher {
    #[command(after_help(stencila_publish_zenodo::AFTER_HELP))]
    #[command(after_long_help(stencila_publish_zenodo::AFTER_LONG_HELP))]
    Zenodo(Box<stencila_publish_zenodo::Cli>),
    Ghost(Box<stencila_publish_ghost::Cli>),
    Stencila(Box<stencila_publish_stencila::Cli>),
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
