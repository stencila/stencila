use std::{path::PathBuf, process::exit};

use common::{
    clap::{self, Parser},
    eyre::Result,
};
use document::{CommandWait, Document};

/// Lint a document
#[derive(Debug, Parser)]
pub struct Cli {
    /// The file to lint
    file: PathBuf,

    /// Format the file if necessary
    #[arg(long)]
    format: bool,

    /// Fix any linting issues
    #[arg(long)]
    fix: bool,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self { file, .. } = self;

        let doc = Document::open(&file).await?;
        doc.lint(self.format, self.fix, CommandWait::Yes).await?;

        if self.format || self.fix {
            doc.save(CommandWait::Yes).await?;
        }

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
