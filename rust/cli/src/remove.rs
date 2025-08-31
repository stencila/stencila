use std::{env::current_dir, path::PathBuf};

use cli_utils::color_print::cstr;
use common::{
    clap::{self, Parser},
    eyre::Result,
    tracing,
};
use dirs::closest_stencila_dir;
use document::Document;

/// Remove documents from the workspace database
#[derive(Debug, Parser)]
#[clap(alias = "rm")]
#[command(after_long_help = REMOVE_AFTER_LONG_HELP)]
pub struct Remove {
    /// The document to remove from the workspace database
    #[arg(num_args = 1.., required = true)]
    documents: Vec<String>,
}

pub static REMOVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove a document from workspace database</dim>
  <b>stencila remove</> <g>document.md</>

  <dim># Remove multiple documents</dim>
  <b>stencila remove</> <g>*.md</> <g>docs/*.md</>

  <dim># Use the rm alias</dim>
  <b>stencila rm</> <g>old-document.md</>

<bold><b>Note</b></bold>
  This removes documents from the workspace database
  but does not delete the actual files. The files
  will no longer be indexed or queryable.
"
);

impl Remove {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let Some(first_doc) = self.documents.first() else {
            tracing::warn!("No documents provided");
            return Ok(());
        };

        let first_path = PathBuf::from(first_doc);
        let base_path = if first_path.exists() {
            first_path
        } else {
            current_dir()?
        };

        let stencila_dir = closest_stencila_dir(&base_path, false).await?;
        Document::remove_docs(&stencila_dir, &self.documents).await
    }
}
