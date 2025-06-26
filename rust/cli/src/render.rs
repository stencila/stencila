use std::{path::PathBuf, process::exit};

use cli_utils::{Code, ToStdout};
use common::{
    clap::{self, Parser},
    eyre::Result,
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

    /// The paths of the output files
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
}

impl Cli {
    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        let Self {
            input,
            outputs,
            decode_options,
            execute_options,
            encode_options,
            strip_options,
            ignore_errors,
            no_store,
        } = self;

        let decode_options = decode_options.build(Some(&input), StripOptions::default());

        let doc = Document::open(&input, Some(decode_options)).await?;
        doc.compile().await?;
        doc.execute(execute_options).await?;
        let (errors, ..) = doc.diagnostics_print().await?;

        if !no_store {
            doc.store().await?;
        }

        if errors > 0 && !ignore_errors {
            eprintln!("💣  Errors while executing `{}`", input.display());
            exit(1);
        }

        if outputs.is_empty() {
            if let Some(format) = &encode_options.to {
                let format = Format::from_name(format);
                let content = doc
                    .dump(
                        format.clone(),
                        Some(codecs::EncodeOptions {
                            render: Some(true),
                            ..encode_options.build(
                                Some(&input),
                                None,
                                Format::Markdown,
                                strip_options,
                            )
                        }),
                    )
                    .await?;
                Code::new(format, &content).to_stdout();
            } else {
                preview::Cli::new(input).run().await?;
            }

            return Ok(());
        }

        for output in outputs {
            doc.export(
                &output,
                Some(codecs::EncodeOptions {
                    render: Some(true),
                    ..encode_options.build(
                        Some(&input),
                        Some(&output),
                        Format::Markdown,
                        strip_options.clone(),
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
