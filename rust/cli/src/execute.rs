use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::{Result, bail};

use stencila_cli_utils::{color_print::cstr, message};
use stencila_dirs::closest_workspace_dir;
use stencila_document::Document;
use stencila_node_execute::ExecuteOptions;

use crate::options::{DecodeOptions, StripOptions};

/// Execute a document
#[derive(Debug, Parser)]
#[command(alias = "exec", after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the document to execute
    input: PathBuf,

    /// Do not save the document after executing it
    #[arg(long)]
    no_save: bool,

    /// Cache the document after executing it
    #[arg(long)]
    cache: bool,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[clap(flatten)]
    execute_options: ExecuteOptions,

    /// Arguments passed unchanged to a traced Python script
    #[arg(last = true, allow_hyphen_values = true)]
    arguments: Vec<String>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Execute a Stencila Markdown document</dim>
  <b>stencila execute</b> <g>report.smd</g>

  <dim># Execute and cache a document</dim>
  <b>stencila execute</b> <g>temp.md</g> <c>--cache</c>

  <dim># Force re-execution of all code</dim>
  <b>stencila execute</b> <g>cached.ipynb</g> <c>--force-all</c>

  <dim># Execute using the shorthand alias</dim>
  <b>stencila exec</b> <g>script.r</g>

  <dim># Trace runtime dependencies of a Python script</dim>
  <b>stencila execute</b> <c>--trace</c> <g>script.py</g> -- <g>arg1</g> <g>arg2</g>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        if self.execute_options.trace
            && self
                .input
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("py"))
        {
            let workspace = closest_workspace_dir(&self.input, false).await?;
            let cache_dir = workspace.join(".stencila/cache/runtime");
            let status =
                stencila_kernels::trace_python_script(&self.input, &self.arguments, cache_dir)
                    .await?;
            if !status.success() {
                exit(status.code().unwrap_or(1));
            }
            message!(
                "🚀 Successfully executed and traced `{}`",
                self.input.display()
            );
            return Ok(());
        }
        if !self.arguments.is_empty() {
            bail!("arguments after `--` are supported only for traced Python scripts");
        }
        let mut execute_options = self.execute_options;
        if execute_options.trace {
            let workspace = closest_workspace_dir(&self.input, false).await?;
            let scope = self
                .input
                .canonicalize()
                .ok()
                .and_then(|path| path.strip_prefix(&workspace).ok().map(PathBuf::from))
                .or_else(|| self.input.file_name().map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from("document"));
            execute_options.trace_workspace = Some(workspace);
            execute_options.trace_scope = Some(
                scope
                    .to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "/"),
            );
        }

        let decode_options = self
            .decode_options
            .build(Some(&self.input), StripOptions::default());

        let doc = Document::open(&self.input, Some(decode_options)).await?;
        doc.execute(execute_options).await?;
        let (errors, warnings, ..) = doc.diagnostics_print().await?;

        if !self.no_save {
            doc.save().await?;
        }

        if self.cache {
            doc.store().await?;
        }

        let input = self.input.display();

        if errors > 0 {
            message!("💥  Errors while executing `{}`", input);
            exit(1);
        } else if warnings > 0 {
            message!("⚠️  Warnings while executing `{}`", input)
        } else {
            message!("🚀 Successfully executed `{}`", input)
        }

        Ok(())
    }
}
