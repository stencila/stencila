use std::{env::current_dir, path::PathBuf};

use cli_utils::color_print::cstr;
use common::{
    clap::{self, Parser},
    eyre::Result,
    tracing,
};
use dirs::closest_stencila_dir;
use document::Document;

use crate::options::{DecodeOptions, StripOptions};

/// Add documents to the workspace database
#[derive(Debug, Parser)]
#[command(after_long_help = ADD_AFTER_LONG_HELP)]
pub struct Cli {
    /// The documents to add to the workspace database
    #[arg(num_args = 1.., required = true)]
    documents: Vec<String>,

    #[command(flatten)]
    decode_options: DecodeOptions,

    /// The tool to use for decoding inputs
    ///
    /// Only supported for formats that use alternative external tools for decoding and ignored otherwise.
    #[arg(long, alias = "to-tool")]
    tool: Option<String>,

    /// Arguments to pass through to the tool using for decoding
    ///
    /// Only supported for formats that use external tools for decoding and ignored otherwise.
    #[arg(last = true, allow_hyphen_values = true)]
    tool_args: Vec<String>,

    /// Do not canonicalize the document
    #[arg(long)]
    no_canonicalize: bool,

    /// Do not split document paragraphs into sentences
    #[arg(long)]
    no_sentencize: bool,
}

pub static ADD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Add a single document to workspace database</dim>
  <b>stencila add</> <g>document.md</>

  <dim># Add multiple local Markdown documents</dim>
  <b>stencila add</> <g>*.md</> <g>docs/*.md</>

  <dim># Add all local Markdown documents</dim>
  <b>stencila add</> <g>**/*.md</>

  <dim># Add a bioRxiv preprint using its DOI</dim>
  <b>stencila add</> <g>https://doi.org/10.1101/2021.11.24.469827</>

  <dim># Add specific pages from a PDF document</dim>
  <b>stencila add</> <g>report.pdf</> <c>--pages</> <g>1,3,5-10</>

  <dim># Add PDF excluding cover and appendix pages</dim>
  <b>stencila add</> <g>book.pdf</> <c>--pages</> <g>2-</> <c>--exclude-pages</> <g>50-</>

  <dim># Add only even pages from a document</dim>
  <b>stencila add</> <g>manuscript.pdf</> <c>--pages</> <g>even</>

<bold><b>Note</b></bold>
  This adds documents to the workspace database for
  indexing and querying. Files must be within the
  workspace directory to be added. Page selection
  options are available for multi-page formats like PDF.
"
);

impl Cli {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let Some(first_doc) = self.documents.first() else {
            tracing::warn!("No documents provided");
            return Ok(());
        };

        let first_path = PathBuf::from(first_doc);

        let base_path = if first_path.exists() {
            first_path.clone()
        } else {
            current_dir()?
        };
        let stencila_dir = closest_stencila_dir(&base_path, true).await?;

        let decode_options: codecs::DecodeOptions = self
            .decode_options
            .build(Some(&first_path), StripOptions::default())
            .with_tool(self.tool, self.tool_args);

        Document::add_docs(
            &stencila_dir,
            &self.documents,
            Some(decode_options),
            !self.no_canonicalize,
            !self.no_sentencize,
        )
        .await
    }
}
