use std::path::PathBuf;

use common::{
    clap::{self, Parser, ValueEnum},
    eyre::Result,
};
use document::Document;
use format::Format;

/// Create a new document with sidecar file
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the document to create
    path: PathBuf,

    /// Overwrite the document, and any sidecar file, if they already exist
    #[arg(long, short)]
    force: bool,

    /// The format of the sidecar file
    #[arg(long, short)]
    sidecar: Option<SidecarFormat>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SidecarFormat {
    #[clap(name = "json.zip", alias = "json-zip")]
    JsonZip,
    Json,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let sidecar = self.sidecar.map(|format| match format {
            SidecarFormat::JsonZip => Format::JsonZip,
            SidecarFormat::Json => Format::Json,
        });
        Document::create(&self.path, self.force, sidecar).await?;

        Ok(())
    }
}
