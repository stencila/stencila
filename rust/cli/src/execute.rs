use std::path::PathBuf;

use cli_utils::{Code, ToStdout};
use codecs::LossesResponse;
use common::{
    clap::{self, Parser},
    eyre::Result,
};
use document::{CommandWait, Document, SaveDocumentSidecar, SaveDocumentSource};
use format::Format;
use node_execute::ExecuteOptions;

use crate::options::{EncodeOptions, StripOptions};

/// Execute a document
#[derive(Debug, Parser)]
#[command(alias = "exec")]
pub struct Cli {
    /// The path of the file to execute
    ///
    /// If not supplied the input content is read from `stdin`.
    input: PathBuf,

    /// The path of the file to write the executed document to
    ///
    /// If not supplied the output content is written to `stdout`.
    output: Option<PathBuf>,

    /// The format to encode to (or codec to use)
    ///
    /// Defaults to inferring the format from the file name extension
    /// of the `output`. If no `output` is supplied, defaults to JSON.
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

    /// Arguments to pass through to any CLI tool delegated to for encoding to the output format (e.g. Pandoc)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    passthrough_args: Vec<String>,
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
            passthrough_args,
        } = self;

        let doc = Document::open(&input).await?;
        doc.compile(CommandWait::Yes).await?;
        doc.execute(execute_options, CommandWait::Yes).await?;

        if !no_save {
            doc.save_with(
                CommandWait::Yes,
                SaveDocumentSource::Yes,
                SaveDocumentSidecar::Yes,
            )
            .await?;
        }

        if output.is_some() || to.is_some() {
            let encode_options = encode_options.build(
                Some(input.as_ref()),
                output.as_deref(),
                to,
                Format::Json,
                strip_options,
                LossesResponse::Debug,
                passthrough_args.clone(),
            );

            let content = doc
                .export(output.as_deref(), Some(encode_options.clone()))
                .await?;

            if !content.is_empty() {
                Code::new(encode_options.format.unwrap_or_default(), &content).to_stdout();
            }
        }

        Ok(())
    }
}

/*

    /// Render a document
    ///
    /// Equivalent to the `execute` command with the `--render` flag.
    #[command()]
    Render {
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
        #[arg(long, short)]
        to: Option<String>,

        #[clap(flatten)]
        execute_options: ExecuteOptions,

        #[command(flatten)]
        encode_options: EncodeOptions,

        #[command(flatten)]
        strip_options: StripOptions,
    },

            Command::Render {
                input,
                output,
                to,
                execute_options,
                encode_options,
                strip_options,
            } => {
                let doc = Document::open(&input).await?;
                doc.compile(CommandWait::Yes).await?;
                doc.execute(execute_options, CommandWait::Yes).await?;

                let mut encode_options = encode_options.build(
                    Some(input.as_ref()),
                    output.as_deref(),
                    to,
                    Format::Markdown,
                    strip_options,
                    LossesResponse::Debug,
                );
                encode_options.render = Some(true);

                let content = doc
                    .export(output.as_deref(), Some(encode_options.clone()))
                    .await?;

                if !content.is_empty() {
                    Code::new(encode_options.format.unwrap_or_default(), &content).to_stdout();
                }
            }
*/
