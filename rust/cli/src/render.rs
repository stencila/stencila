use std::{path::PathBuf, process::exit};

use cli_utils::{Code, ToStdout};
use common::{
    clap::{self, Parser},
    eyre::{bail, eyre, Result},
};
use document::Document;
use format::Format;
use node_execute::ExecuteOptions;

use crate::{
    options::{DecodeOptions, EncodeOptions, StripOptions},
    preview,
};

/// Render a document
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the document to render
    input: PathBuf,

    /// The paths of desired output files
    ///
    /// If no outputs are supplied, and the `--to` format option is not used,
    /// the document will be rendered in a browser window. If no outputs are
    /// supplied and the `--to` option is used the document will be rendered
    /// to `stdout` in that format.
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

    /// Document arguments
    ///
    /// The name of each argument is matched against the document's parameters.
    /// If a match is found, then the argument value is coerced to the expected
    /// type of the parameter. If no corresponding parameter is found, then the
    /// argument is parsed as JSON and set as a variable in the document's default
    /// kernel (usually the first programming language used in the document).
    #[arg(last = true, allow_hyphen_values = true)]
    arguments: Vec<String>,
}

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

        let decode_options = self
            .decode_options
            .build(Some(&input), StripOptions::default());

        let doc = Document::open(&input, Some(decode_options)).await?;
        doc.compile().await?;
        doc.call(&arguments, self.execute_options.clone()).await?;
        let (errors, ..) = doc.diagnostics_print().await?;

        if !self.no_store {
            doc.store().await?;
        }

        if errors > 0 && !self.ignore_errors {
            eprintln!("💥  Errors while executing `{}`", input.display());
            exit(1);
        }

        if outputs.is_empty() {
            if let Some(format) = &self.encode_options.to {
                let format = Format::from_name(format);
                let content = doc
                    .dump(
                        format.clone(),
                        Some(codecs::EncodeOptions {
                            render: Some(true),
                            ..self.encode_options.build(
                                Some(&input),
                                None,
                                Format::Markdown,
                                self.strip_options,
                            )
                        }),
                    )
                    .await?;
                Code::new(format, &content).to_stdout();
            } else {
                preview::Cli::new(input.into()).run().await?;
            }

            return Ok(());
        }

        for output in outputs {
            doc.export(
                &output,
                Some(codecs::EncodeOptions {
                    render: Some(true),
                    ..self.encode_options.build(
                        Some(&input),
                        Some(&output),
                        Format::Markdown,
                        self.strip_options.clone(),
                    )
                }),
            )
            .await?;
            eprintln!(
                "📑 Successfully rendered `{}` to `{}`",
                input.display(),
                output.display()
            )
        }

        Ok(())
    }
}
