use std::{path::PathBuf, process::exit};

use common::{
    clap::{self, Parser},
    eyre::Result,
};
use document::{CommandWait, Document};

/// Lint one or more documents
#[derive(Debug, Parser)]
pub struct Cli {
    /// The files to lint
    files: Vec<PathBuf>,

    /// Format the file if necessary
    #[arg(long)]
    format: bool,

    /// Fix any linting issues
    #[arg(long)]
    fix: bool,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let mut files_with_issues = 0;
        let mut count_of_issues = 0;
        for file in self.files {
            let doc = Document::open(&file).await?;
            doc.lint(self.format, self.fix, CommandWait::Yes).await?;

            if self.format || self.fix {
                doc.save(CommandWait::Yes).await?;
            }

            let count = doc.diagnostics().await?;
            if count > 0 {
                count_of_issues += count;
                files_with_issues += 1;
            }
        }

        #[allow(clippy::print_stderr)]
        if files_with_issues == 0 {
            eprintln!("🎉 No problems found")
        } else {
            eprintln!(
                "⚠️  {count_of_issues} issue{} found in {files_with_issues} file{}",
                if count_of_issues > 1 { "s" } else { "" },
                if files_with_issues > 1 { "s" } else { "" }
            );

            exit(1)
        }

        Ok(())
    }
}
