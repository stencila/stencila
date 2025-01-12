use std::path::PathBuf;

use common::{
    clap::{self, Parser, ValueEnum},
    eyre::Result,
};
use document::Document;
use schema::NodeType;

/// Create a new document with sidecar file
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the document to create
    path: PathBuf,

    /// Overwrite the document, and any sidecar file, if they already exist
    #[arg(long, short)]
    force: bool,

    /// The type of document to create
    #[arg(long, short, default_value = "article")]
    r#type: RootType,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum RootType {
    #[clap(alias = "article")]
    Article,
    #[clap(alias = "chat")]
    Chat,
    #[clap(alias = "prompt")]
    Prompt,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let node_type = match self.r#type {
            RootType::Article => NodeType::Article,
            RootType::Chat => NodeType::Chat,
            RootType::Prompt => NodeType::Prompt,
        };
        Document::create(&self.path, self.force, node_type).await?;

        Ok(())
    }
}
