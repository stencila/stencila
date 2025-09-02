use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::Result;

use cli_utils::{AsFormat, Code, ToStdout, color_print::cstr};
use document::Document;

/// Lint one or more documents
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The files to lint
    files: Vec<PathBuf>,

    /// Format the file if necessary
    #[arg(long)]
    format: bool,

    /// Fix any linting issues
    #[arg(long)]
    fix: bool,

    /// Do not store the document after formatting and/or fixing it
    ///
    /// Only applies when using `--format` or `--fix`, both of which will write a
    /// modified version of the source document back to disk and by default, a new
    /// cache of the document to the store. This flag prevent the store being updated.
    #[arg(long)]
    no_store: bool,

    /// Output any linting diagnostics as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Lint a single document</dim>
  <b>stencila lint</> <g>document.smd</>

  <dim># Lint multiple documents</dim>
  <b>stencila lint</> <g>*.qmd</> <g>docs/*</>

  <dim># Auto-format documents during linting</dim>
  <b>stencila lint</> <g>report.myst</> <c>--format</>

  <dim># Auto-fix linting issues</dim>
  <b>stencila lint</> <g>article.smd</> <c>--fix</>

  <dim># Output diagnostics as YAML</dim>
  <b>stencila lint</> <g>article.myst</> <c>--as</> <g>yaml</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let mut files_with_issues = 0;
        let mut count_of_issues = 0;
        for file in self.files {
            let doc = Document::open(&file, None).await?;
            doc.lint(self.format, self.fix).await?;

            if self.format || self.fix {
                doc.save().await?;

                if !self.no_store {
                    doc.store().await?;
                }
            }

            if let Some(format) = self.r#as.clone() {
                let diagnostics = doc.diagnostics().await;
                count_of_issues += diagnostics.len();
                Code::new_from(format.into(), &diagnostics)?.to_stdout();
            } else {
                let (errors, warnings, advice, ..) = doc.diagnostics_print().await?;
                let all = errors + warnings + advice;
                if all > 0 {
                    count_of_issues += all;
                    files_with_issues += 1;
                }
            }
        }

        if self.r#as.is_some() && count_of_issues > 0 {
            exit(count_of_issues as i32)
        }

        #[allow(clippy::print_stderr)]
        if files_with_issues == 0 {
            eprintln!("🎉 No issues found")
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
