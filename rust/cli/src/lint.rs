use std::{path::PathBuf, process::exit};

use cli_utils::{AsFormat, Code, ToStdout};
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

    /// Output any linting diagnostics as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let mut files_with_issues = 0;
        let mut count_of_issues = 0;
        for file in self.files {
            let doc = Document::open(&file).await?;
            doc.lint(self.format, self.fix, CommandWait::Yes).await?;

            if self.format || self.fix {
                doc.save().await?;
            }

            if let Some(format) = self.r#as.clone() {
                let diagnostics = doc.diagnostics().await;
                count_of_issues += diagnostics.len();
                Code::new_from(format.into(), &diagnostics)?.to_stdout();
            } else {
                let count = doc.diagnostics_print().await?;
                if count > 0 {
                    count_of_issues += count;
                    files_with_issues += 1;
                }
            }
        }

        if self.r#as.is_some() && count_of_issues > 0 {
            exit(count_of_issues as i32)
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
