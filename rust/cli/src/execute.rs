use std::{path::PathBuf, process::exit};

use common::{
    clap::{self, Parser},
    eyre::Result,
};
use document::Document;
use node_execute::ExecuteOptions;

use crate::options::{DecodeOptions, StripOptions};

/// Execute a document
#[derive(Debug, Parser)]
#[command(alias = "exec")]
pub struct Cli {
    /// The path of the document to execute
    input: PathBuf,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[clap(flatten)]
    execute_options: ExecuteOptions,

    /// Do not store the document after executing it
    #[arg(long)]
    no_store: bool,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self {
            input,
            decode_options,
            execute_options,
            no_store,
        } = self;

        let decode_options =
            decode_options.build(Some(&input), StripOptions::default(), None, Vec::new());

        let doc = Document::open(&input, Some(decode_options)).await?;
        doc.compile().await?;
        doc.execute(execute_options).await?;
        let (errors, warnings, ..) = doc.diagnostics_print().await?;

        if !no_store {
            doc.store().await?;
        }

        #[allow(clippy::print_stderr)]
        if errors > 0 {
            eprintln!("💣  Errors while executing `{}`", input.display());
            exit(1);
        } else if warnings > 0 {
            eprintln!("⚠️ Warnings while executing `{}`", input.display())
        } else {
            eprintln!("🚀 Successfully executed `{}`", input.display())
        }

        Ok(())
    }
}
