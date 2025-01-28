use std::{path::PathBuf, process::exit};

use common::{
    clap::{self, Parser},
    eyre::Result,
};
use document::{CommandWait, Document};

/// Lint a document
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the file to execute
    ///
    /// If not supplied the input content is read from `stdin`.
    input: PathBuf,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self { input, .. } = self;

        let doc = Document::open(&input).await?;
        doc.lint(false, false, CommandWait::Yes).await?;

        let some = doc.diagnostics().await?;

        #[allow(clippy::print_stderr)]
        if !some {
            eprintln!("🎉 No problems found")
        } else {
            exit(1)
        }

        Ok(())
    }
}
