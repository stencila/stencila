use std::path::PathBuf;

use cli_utils::{Code, ToStdout};
use codecs::LossesResponse;
use common::{
    clap::{self, Parser},
    eyre::Result,
};
use document::Document;
use format::Format;
use node_execute::ExecuteOptions;

use crate::options::{EncodeOptions, StripOptions};

/// Render a document
///
/// Equivalent to the `execute` command with the `--render` flag.
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the file to render
    ///
    /// If not supplied the input content is read from `stdin`.
    input: PathBuf,

    /// The path of the file to write the rendered document to
    ///
    /// If not supplied the output content is written to `stdout`.
    output: Option<PathBuf>,

    /// The format to encode to (or codec to use)
    ///
    /// Defaults to inferring the format from the file name extension
    /// of the `output`. If no `output` is supplied, defaults to Markdown.
    /// See `stencila codecs list` for available formats.
    #[arg(long, short)]
    to: Option<String>,

    #[clap(flatten)]
    execute_options: ExecuteOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    #[command(flatten)]
    strip_options: StripOptions,

    /// Do not save the document after compiling it
    #[arg(long)]
    no_save: bool,

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
            to,
            execute_options,
            encode_options,
            strip_options,
            no_save,
            tool,
            tool_args,
        } = self;

        let doc = Document::open(&input, None).await?;
        doc.compile().await?;
        doc.execute(execute_options).await?;
        doc.diagnostics_print().await?;

        if !no_save {
            doc.save().await?;
        }

        let encode_options = encode_options.build(
            Some(input.as_ref()),
            output.as_deref(),
            to.clone(),
            Format::Markdown,
            strip_options,
            LossesResponse::Debug,
            tool,
            tool_args,
        );

        if let Some(dest) = &output {
            doc.export(dest, Some(encode_options)).await?;
        } else if let Some(format) = to {
            let format = Format::from_name(&format);
            let content = doc.dump(format.clone(), Some(encode_options)).await?;
            Code::new(format, &content).to_stdout();
        }

        Ok(())
    }
}
