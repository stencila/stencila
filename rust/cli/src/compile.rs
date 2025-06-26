use std::{path::PathBuf, process::exit};

use common::{
    clap::{self, Parser},
    eyre::Result,
};
use document::Document;

use crate::options::{DecodeOptions, StripOptions};

/// Compile a document
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the document to compile
    input: PathBuf,

    /// Do not store the document after compiling it
    #[arg(long)]
    no_store: bool,

    #[command(flatten)]
    decode_options: DecodeOptions,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self {
            input,
            decode_options,
            no_store,
            ..
        } = self;

        let decode_options = decode_options.build(Some(&input), StripOptions::default());

        let doc = Document::open(&input, Some(decode_options)).await?;
        doc.compile().await?;
        let (errors, warnings, ..) = doc.diagnostics_print().await?;

        if !no_store {
            doc.store().await?;
        }

        #[allow(clippy::print_stderr)]
        if errors > 0 {
            eprintln!("💥  Errors while compiling `{}`", input.display());
            exit(1)
        } else if warnings > 0 {
            eprintln!("⚠️  Warnings while compiling `{}`", input.display())
        } else {
            eprintln!("🛠️  Successfully compiled `{}`", input.display())
        }

        Ok(())
    }
}
