use std::path::PathBuf;

use clap::{self, Parser, ValueEnum};
use eyre::Result;

use cli_utils::color_print::cstr;
use document::Document;
use schema::NodeType;

/// Create a new, tracked, document
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the document to create
    path: PathBuf,

    /// Overwrite the document, if it already exists
    #[arg(long, short)]
    force: bool,

    /// The type of document to create
    #[arg(long, short, default_value = "article")]
    r#type: RootType,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a new article (default)</dim>
  <b>stencila new</> <g>my-article.md</>

  <dim># Create a new chat document</dim>
  <b>stencila new</> <g>conversation.md</> <c>--type</> <g>chat</>

  <dim># Create a new AI prompt</dim>
  <b>stencila new</> <g>template.md</> <c>--type</> <g>prompt</>

  <dim># Create a document in a subdirectory</dim>
  <b>stencila new</> <g>docs/report.md</>

  <dim># Overwrite an existing document</dim>
  <b>stencila new</> <g>existing.md</> <c>--force</>
"
);

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
