use std::path::PathBuf;

use cli_utils::{Code, ToStdout};
use common::{
    clap::{self, Parser},
    eyre::Result,
};
use document::Document;
use format::Format;

use crate::options::{EncodeOptions, StripOptions};

/// Compile a document
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the file to execute
    ///
    /// If not supplied the input content is read from `stdin`.
    input: PathBuf,

    /// The path of the file to write the executed document to
    ///
    /// If not supplied the output content is written to `stdout`
    /// in the format specified by the `--to` option (defaulting to JSON).
    output: Option<PathBuf>,

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
            encode_options,
            strip_options,
            no_save,
            tool,
            tool_args,
            ..
        } = self;

        let doc = Document::open(&input, None).await?;
        doc.compile().await?;
        doc.diagnostics_print().await?;

        if !no_save {
            doc.save().await?;
        }

        let to = encode_options.to.clone();
        if output.is_some() || to.is_some() {
            let encode_options = Some(encode_options.build(
                Some(input.as_ref()),
                output.as_deref(),
                Format::Json,
                strip_options,
                tool,
                tool_args,
            ));

            if let Some(dest) = &output {
                doc.export(dest, encode_options).await?;
            } else if let Some(format) = to {
                let format = Format::from_name(&format);
                let content = doc.dump(format.clone(), encode_options).await?;
                Code::new(format, &content).to_stdout();
            }
        }

        Ok(())
    }
}
