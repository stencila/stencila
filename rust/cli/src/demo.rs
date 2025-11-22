use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::Result;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_document::{Document, demo::DemoOptions};
use stencila_node_execute::ExecuteOptions;

/// Run a terminal demonstration from a document
#[derive(Debug, Parser)]
#[command(after_long_help = DEMO_AFTER_LONG_HELP)]
pub struct Demo {
    /// The path of the document to demo
    input: PathBuf,

    #[clap(flatten)]
    demo_options: DemoOptions,

    /// Do not execute the document before running the demo
    #[arg(long)]
    no_execute: bool,

    /// Do not store the document after executing it
    #[arg(long, conflicts_with = "no_execute")]
    no_store: bool,

    #[clap(flatten)]
    execute_options: ExecuteOptions,
}

impl Demo {
    #[allow(clippy::print_stderr)]
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        let doc = Document::open(&self.input, None).await?;

        let ignore_errors = self.execute_options.ignore_errors;

        if !self.no_execute {
            doc.compile().await?;
            doc.execute(self.execute_options).await?;

            let (errors, ..) = doc.diagnostics_print().await?;

            if errors > 0 {
                if ignore_errors {
                    message!("▶️  Ignoring execution errors")
                } else {
                    message!(
                        "🛑 Stopping due to execution errors (you can use `--ignore-errors` to continue demo regardless)"
                    );
                    exit(1)
                }
            }

            if !self.no_store {
                doc.store().await?;
            }
        }

        doc.demo(self.demo_options).await
    }
}

pub static DEMO_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Demo a document in the terminal (uses natural preset by default)</dim>
  <b>stencila demo</> <g>document.md</>

  <dim># Record a demo to an animated GIF</dim>
  <b>stencila demo</> <g>document.md</> <g>demo.gif</>

  <dim># Use fast preset for quick, smooth typing</dim>
  <b>stencila demo</> <g>document.md</> <c>--preset</> <g>fast</>

  <dim># Use fast preset but add some typing variance</dim>
  <b>stencila demo</> <g>document.md</> <c>--preset</> <g>fast</> <c>--speed-variance</> <g>0.2</>

  <dim># Use fast preset but extend the maximum duration of running times</dim>
  <b>stencila demo</> <g>document.md</> <c>--preset</> <g>fast</> <c>--min-running</> <g>2000</> <c>--max-running</> <g>4000</>

  <dim># Use instant preset for immediate results</dim>
  <b>stencila demo</> <g>document.md</> <c>--preset</> <g>instant</>

  <dim># Disable syntax highlighting for code blocks</dim>
  <b>stencila demo</> <g>document.md</> <c>--no-highlighting</>

  <dim># Demo only specific slides (slides are delimited by ***)</dim>
  <b>stencila demo</> <g>document.md</> <c>--slides</> <g>2-4</>

  <dim># Demo multiple slide ranges</dim>
  <b>stencila demo</> <g>document.md</> <c>--slides</> <g>\"1,3-5,7-\"</>
"
);
