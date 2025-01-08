use std::path::PathBuf;

use common::{
    clap::{self, Parser},
    eyre::Result,
};

/// Publish to Ghost
#[derive(Debug, Parser)]
pub struct Cli {
    /// Path to the file or directory to publish
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// The Ghost domain
    #[arg(long, env = "STENCILA_GHOST_DOMAIN")]
    domain: bool,

    /// The Ghost Admin API key
    #[arg(long, env = "STENCILA_GHOST_KEY")]
    key: bool,

    /// Create a post
    #[arg(long, conflicts_with = "page")]
    post: bool,

    /// Create a page
    #[arg(long, conflicts_with = "post")]
    page: bool,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        Ok(())
    }
}
