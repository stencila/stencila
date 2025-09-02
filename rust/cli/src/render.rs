use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::{Result, bail, eyre};

use stencila_ask::{Answer, AskLevel, AskOptions, ask_with};
use stencila_cli_utils::{Code, ToStdout, color_print::cstr};
use stencila_document::Document;
use stencila_format::Format;
use stencila_node_execute::ExecuteOptions;

use crate::{
    options::{DecodeOptions, EncodeOptions, StripOptions},
    preview,
};

/// Render a document
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the document to render
    ///
    /// If not supplied, or if "-", the input content is read from `stdin`
    /// and assumed to be Markdown (but can be specified with the `--from` option).
    /// Note that the Markdown parser should handle alternative flavors so
    /// it may not be necessary to use the `--from` option for MyST, Quarto or
    /// Stencila Markdown.
    input: Option<PathBuf>,

    /// The paths of desired output files
    ///
    /// If an input was supplied, but no outputs, and the `--to` format option
    /// is not used, the document will be rendered in a browser window.
    /// If no outputs are supplied and the `--to` option is used the document
    /// will be rendered to `stdout` in that format.
    outputs: Vec<PathBuf>,

    /// Ignore any errors while executing document
    #[arg(long)]
    ignore_errors: bool,

    /// Do not store the document after executing it
    #[arg(long)]
    no_store: bool,

    #[clap(flatten)]
    execute_options: ExecuteOptions,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    #[command(flatten)]
    strip_options: StripOptions,

    /// Arguments to pass to the document
    ///
    /// The name of each argument is matched against the document's parameters.
    /// If a match is found, then the argument value is coerced to the expected
    /// type of the parameter. If no corresponding parameter is found, then the
    /// argument is parsed as JSON and set as a variable in the document's default
    /// kernel (usually the first programming language used in the document).
    #[arg(last = true, allow_hyphen_values = true)]
    arguments: Vec<String>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Render a document and preview in browser</dim>
  <b>stencila render</b> <g>document.smd</g>

  <dim># Render to a specific output format</dim>
  <b>stencila render</b> <g>report.md</g> <g>report.docx</g>

  <dim># Render to multiple formats</dim>
  <b>stencila render</b> <g>analysis.md</g> <g>output.html</g> <g>output.pdf</g>

  <dim># Render from stdin to stdout</dim>
  <b>echo</b> <y>\"# Hello\"</y> | <b>stencila render</b> <c>--to</c> <g>html</g>

  <dim># Render with document parameters</dim>
  <b>stencila render</b> <g>template.md</g> <g>output.html</g> -- <c>--name</c>=<y>\"John\"</y> <c>--year</c>=<g>2024</g>

  <dim># Render ignoring execution errors</dim>
  <b>stencila render</b> <g>notebook.md</g> <g>report.pdf</g> <c>--ignore-errors</c>

  <dim># Render without updating the document store</dim>
  <b>stencila render</b> <g>temp.md</g> <g>output.html</g> <c>--no-store</c>
"
);

impl Cli {
    /// Parse document arguments into name/value pairs
    fn arguments(&self) -> Result<Vec<(&str, &str)>> {
        let mut parsed = Vec::new();
        let mut index = 0;
        while index < self.arguments.len() {
            let current = &self.arguments[index];

            if current.starts_with("--") {
                // Handle --name=value format
                if let Some((name, value)) = current.split_once('=') {
                    let name = name
                        .strip_prefix("--")
                        .ok_or_else(|| eyre!("Invalid argument format: '{current}'"))?;

                    if name.is_empty() || value.is_empty() {
                        bail!(
                            "Invalid argument format: '{current}'. Name and value cannot be empty"
                        );
                    }

                    parsed.push((name, value));
                    index += 1;
                }
                // Handle --name value format
                else {
                    let name = current
                        .strip_prefix("--")
                        .ok_or_else(|| eyre!("Invalid argument format: '{current}'"))?;

                    if name.is_empty() {
                        bail!("Invalid argument format: '{current}'. Name cannot be empty");
                    }

                    // Check if there's a next argument for the value
                    if index + 1 >= self.arguments.len() {
                        bail!("Parameter '{current}' requires a value");
                    }

                    let value = &self.arguments[index + 1];

                    // Make sure the value doesn't look like another argument
                    if value.starts_with("--") {
                        bail!(
                            "Parameter '{current}' requires a value, but found another argument '{value}'"
                        );
                    }

                    parsed.push((name, value));
                    index += 2;
                }
            }
            // Handle name=value format (no dashes)
            else if current.contains('=') {
                if let Some((name, value)) = current.split_once('=') {
                    let name = name.trim();
                    let value = value.trim();

                    if name.is_empty() || value.is_empty() {
                        bail!(
                            "Invalid argument format: '{current}'. Name and value cannot be empty"
                        );
                    }

                    parsed.push((name, value));
                    index += 1;
                } else {
                    bail!("Invalid argument format: '{current}'");
                }
            }
            // Reject bare values
            else {
                bail!(
                    "Invalid argument '{current}'. Use 'name=value', '--name=value', or '--name value' format"
                );
            }
        }

        Ok(parsed)
    }

    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        let input = &self.input;
        let outputs = &self.outputs;
        let arguments = self.arguments()?;

        let input_path = input.clone().unwrap_or_else(|| PathBuf::from("-"));
        let input_stdin = input_path == PathBuf::from("-");
        let input_display = if input_stdin {
            "stdin".to_string()
        } else {
            input_path.to_string_lossy().to_string()
        };

        let doc = if input_stdin {
            let mut decode_options = self.decode_options.build(None, StripOptions::default());
            if decode_options.format.is_none() {
                decode_options.format = Some(Format::Markdown)
            }

            let root = stencila_codecs::from_stdin(Some(decode_options.clone())).await?;
            Document::from(root, None, Some(decode_options)).await?
        } else {
            let decode_options = self
                .decode_options
                .build(input.as_deref(), StripOptions::default());
            Document::open(&input_path, Some(decode_options)).await?
        };

        doc.compile().await?;
        doc.call(&arguments, self.execute_options.clone()).await?;
        let (errors, ..) = doc.diagnostics_print().await?;

        if !self.no_store && !input_stdin {
            doc.store().await?;
        }

        if errors > 0 {
            if self.ignore_errors {
                eprintln!("▶️  Ignoring execution errors")
            } else if ask_with(
                &format!("Errors while executing `{input_display}`. Continue rendering?"),
                AskOptions {
                    level: AskLevel::Warning,
                    default: Some(Answer::Yes),
                    ..Default::default()
                },
            )
            .await?
            .is_yes()
            {
                eprintln!("▶️  Tip: use `--ignore-errors` to continue without being asked")
            } else {
                eprintln!(
                    "🛑 Stopping due to execution errors (Tip: use `--ignore-errors` to continue without being asked)"
                );
                exit(1)
            }
        }

        if outputs.is_empty() {
            if let Some(format) = self
                .encode_options
                .to
                .as_ref()
                .map(|format| Format::from_name(format))
                .or_else(|| input_stdin.then_some(Format::Markdown))
            {
                // If a `--to` format was supplied, or input was stdin (i.e. no
                // path to review in the browser) then dump to console
                let content = doc
                    .dump(
                        format.clone(),
                        Some(stencila_codecs::EncodeOptions {
                            render: Some(true),
                            ..self.encode_options.build(
                                input.as_deref(),
                                None,
                                Format::Markdown,
                                self.strip_options,
                            )
                        }),
                    )
                    .await?;
                Code::new(format, &content).to_stdout();
            } else if let Some(input) = input {
                // Otherwise render the path in the browser
                preview::Cli::new(input.to_path_buf()).run().await?;
            }

            return Ok(());
        }

        for output in outputs {
            let completed = doc
                .export(
                    output,
                    Some(stencila_codecs::EncodeOptions {
                        render: Some(true),
                        ..self.encode_options.build(
                            input.as_deref(),
                            Some(output),
                            Format::Markdown,
                            self.strip_options.clone(),
                        )
                    }),
                )
                .await?;

            if completed {
                eprintln!(
                    "📑 Successfully rendered `{input_display}` to `{}`",
                    output.display()
                )
            } else {
                eprintln!("⏭️  Skipped rendering `{input_display}`")
            }
        }

        Ok(())
    }
}
