use std::{path::PathBuf, process::exit};

use cli_utils::{Code, ToStdout};
use codecs::LossesResponse;
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

    /// The path of the rendered document
    output: Option<PathBuf>,

    /// Ignore any errors while executing document
    #[arg(long)]
    ignore_errors: bool,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[clap(flatten)]
    execute_options: ExecuteOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    #[command(flatten)]
    strip_options: StripOptions,

    /// Do not store the document after executing it
    #[arg(long)]
    no_store: bool,

    /// The tool to use to encode the output format
    #[arg(long)]
    tool: Option<String>,

    /// Arguments to pass through to any CLI tool delegated to for encoding to the output format (e.g. Pandoc)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    tool_args: Vec<String>,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self {
            input,
            output,
            decode_options,
            execute_options,
            encode_options,
            strip_options,
            ignore_errors,
            no_store,
            tool,
            tool_args,
        } = self;

        let decode_options =
            decode_options.build(Some(&input), StripOptions::default(), None, Vec::new());

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

        if output.is_none() && encode_options.to.is_none() {
            return preview::Cli::new(input).run().await;
        }

        let mut encode_options = encode_options.build(
            Some(&input),
            output.as_deref(),
            Format::Markdown,
            strip_options,
            tool,
            tool_args,
        );
        encode_options.render = Some(true);
        encode_options.losses = LossesResponse::Debug;

        #[allow(clippy::print_stderr)]
        if let Some(output) = &output {
            doc.export(output, Some(encode_options)).await?;
            eprintln!(
                "📑 Successfully rendered `{}` to `{}`",
                input.display(),
                output.display()
            )
        } else if let Some(to) = encode_options.format.clone() {
            let content = doc.dump(to.clone(), Some(encode_options)).await?;
            Code::new(to, &content).to_stdout();
        }

        Ok(())
    }
}
