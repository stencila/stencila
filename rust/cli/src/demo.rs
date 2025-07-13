use std::{path::PathBuf, process::exit};

use cli_utils::color_print::cstr;
use common::{
    clap::{self, Parser},
    eyre::Result,
    tracing,
};
use document::{demo::DemoOptions, Document};
use node_execute::ExecuteOptions;

/// Run a terminal demonstration from a document
#[derive(Debug, Parser)]
#[command(after_long_help = DEMO_AFTER_LONG_HELP)]
pub struct Demo {
    /// The path of the document to demo
    input: PathBuf,

    #[clap(flatten)]
    demo_options: DemoOptions,

    /// Ignore any errors while executing document
    #[arg(long)]
    ignore_errors: bool,

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

        if !self.no_execute {
            doc.compile().await?;
            doc.execute(self.execute_options).await?;

            let (errors, ..) = doc.diagnostics_print().await?;

            if errors > 0 {
                if self.ignore_errors {
                    eprintln!("▶️  Ignoring execution errors")
                } else {
                    eprintln!("🛑 Stopping due to execution errors (you can use `--ignore-errors` to continue demo regardless)");
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
"
);
