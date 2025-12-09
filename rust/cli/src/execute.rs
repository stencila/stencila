use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::Result;

use stencila_cli_utils::color_print::cstr;
use stencila_document::Document;
use stencila_node_execute::ExecuteOptions;

use crate::options::{DecodeOptions, StripOptions};

/// Execute a document
#[derive(Debug, Parser)]
#[command(alias = "exec", after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the document to execute
    input: PathBuf,

    /// Do not save the document after executing it
    #[arg(long)]
    no_save: bool,

    /// Cache the document after executing it
    #[arg(long)]
    cache: bool,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[clap(flatten)]
    execute_options: ExecuteOptions,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Execute a Stencila Markdown document</dim>
  <b>stencila execute</b> <g>report.smd</g>

  <dim># Execute and cache a document</dim>
  <b>stencila execute</b> <g>temp.md</g> <c>--cache</c>

  <dim># Force re-execution of all code</dim>
  <b>stencila execute</b> <g>cached.ipynb</g> <c>--force-all</c>

  <dim># Execute using the shorthand alias</dim>
  <b>stencila exec</b> <g>script.r</g>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let decode_options = self
            .decode_options
            .build(Some(&self.input), StripOptions::default());

        let doc = Document::open(&self.input, Some(decode_options)).await?;
        doc.compile().await?;
        doc.execute(self.execute_options).await?;
        let (errors, warnings, ..) = doc.diagnostics_print().await?;

        if !self.no_save {
            doc.save().await?;
        }

        if self.cache {
            doc.store().await?;
        }

        let input = self.input.display();

        #[allow(clippy::print_stderr)]
        if errors > 0 {
            eprintln!("💥  Errors while executing `{input}`");
            exit(1);
        } else if warnings > 0 {
            eprintln!("⚠️  Warnings while executing `{input}`")
        } else {
            eprintln!("🚀 Successfully executed `{input}`")
        }

        Ok(())
    }
}
