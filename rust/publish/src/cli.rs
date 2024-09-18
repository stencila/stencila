use std::path::PathBuf;

use codec_swb::SwbCodec;
use common::{
    clap::{self, Parser},
    eyre::Result,
};

/// Publish a document or site
///
/// Currently only supports publishing a single document
/// to the web via Stencila Cloud.
///
/// In the future, it is likely that other publication platforms
/// will be supported.
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path to the document file or site directory to publish
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Key or identifier required by the platform being published to
    #[arg(long, short)]
    key: Option<String>,

    /// Perform a dry run
    #[arg(long)]
    dry_run: bool,

    #[clap(flatten)]
    swb: SwbCodec,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        super::publish_path(&self.path, &self.key, self.dry_run, &self.swb).await
    }
}
