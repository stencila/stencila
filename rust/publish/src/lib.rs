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
    Ghost(publish_ghost::Cli),
    Stencila(publish_stencila::Cli),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        use Publisher::*;
        match self.publisher {
            Ghost(cli) => cli.run().await,
            Stencila(cli) => cli.run().await,
        }
    }
}
