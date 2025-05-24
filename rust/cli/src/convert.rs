use std::path::PathBuf;

use cli_utils::{Code, ToStdout};
use common::{
    clap::{self, Parser},
    eyre::Result,
};
use format::Format;

use crate::options::{DecodeOptions, EncodeOptions, StripOptions};

/// Convert a document to another format
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path of the input file
    ///
    /// If not supplied the input content is read from `stdin`.
    input: Option<PathBuf>,

    /// The path of the output file
    ///
    /// If not supplied the output content is written to `stdout`.
    output: Option<PathBuf>,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    #[command(flatten)]
    strip_options: StripOptions,

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
            encode_options,
            strip_options,
            tool,
            tool_args,
        } = self;

        let decode_options = decode_options.build(None, strip_options.clone(), None, Vec::new());
        let encode_options = encode_options.build(
            input.as_deref(),
            output.as_deref(),
            Format::Json,
            strip_options,
            tool,
            tool_args.clone(),
        );

        let content = codecs::convert(
            input.as_deref(),
            output.as_deref(),
            Some(decode_options),
            Some(encode_options.clone()),
        )
        .await?;

        if !content.is_empty() {
            Code::new(encode_options.format.unwrap_or_default(), &content).to_stdout();
        }

        Ok(())
    }
}
