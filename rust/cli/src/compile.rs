use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::Result;

use stencila_cli_utils::color_print::cstr;
use stencila_document::Document;

use crate::options::{DecodeOptions, StripOptions};

/// Compile a document
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the document to compile
    input: PathBuf,

    /// Do not save the document after compiling it
    #[arg(long)]
    no_save: bool,

    /// Do not store the document after compiling it
    #[arg(long)]
    no_store: bool,

    #[command(flatten)]
    decode_options: DecodeOptions,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Compile a document to check for errors</dim>
  <b>stencila compile</> <g>document.md</>

  <dim># Compile without updating in document store</dim>
  <b>stencila compile</> <g>temp.md</> <c>--no-store</>

<bold><b>Note</b></bold>
  Compiling a document checks for source path errors in
  include and call blocks and prepares the document for
  execution without actually running any code.
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let decode_options = self
            .decode_options
            .build(Some(&self.input), StripOptions::default());

        let doc = Document::open(&self.input, Some(decode_options)).await?;
        doc.compile().await?;
        let (errors, warnings, ..) = doc.diagnostics_print().await?;

        if !self.no_save {
            doc.save().await?;
        }

        if !self.no_store {
            doc.store().await?;
        }

        let input = self.input.display();

        #[allow(clippy::print_stderr)]
        if errors > 0 {
            eprintln!("💥  Errors while compiling `{input}`");
            exit(1)
        } else if warnings > 0 {
            eprintln!("⚠️  Warnings while compiling `{input}`")
        } else {
            eprintln!("🛠️  Successfully compiled `{input}`")
        }

        Ok(())
    }
}
