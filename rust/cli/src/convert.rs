use std::path::PathBuf;

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
    /// If not supplied, or if "-", the input content is read from `stdin`.
    input: Option<PathBuf>,

    /// The paths of the output files
    ///
    /// Each output may be of a different format (inferred from the extension).
    /// If the `--to` format option is used it will apply to all outputs.
    /// If no output paths supplied, or if "-", the output content is written to `stdout`.
    outputs: Vec<PathBuf>,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    #[command(flatten)]
    strip_options: StripOptions,

    /// The tool to use for encoding outputs (e.g. pandoc)
    ///
    /// Only supported for formats that use alternative external tools for encoding and ignored otherwise.
    /// Note: this tool is not used for decoding from the input, only for encoding to the output.
    #[arg(long)]
    tool: Option<String>,

    /// Arguments to pass through to the tool using for encoding
    ///
    /// Only supported for formats that use external tools for encoding and ignored otherwise.
    /// Note: these arguments are not used for decoding from the input, only for encoding to the output.
    #[arg(last = true, allow_hyphen_values = true)]
    tool_args: Vec<String>,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self {
            input,
            outputs,
            decode_options,
            encode_options,
            strip_options,
            tool,
            tool_args,
        } = self;

        let decode_options = decode_options.build(input.as_deref(), strip_options.clone());

        let input_path = input.clone().unwrap_or_else(|| PathBuf::from("-"));
        let node = if input_path == PathBuf::from("-") {
            codecs::from_stdin(Some(decode_options)).await?
        } else {
            codecs::from_path(&input_path, Some(decode_options)).await?
        };

        if outputs.is_empty() || outputs.iter().all(|path| path.to_string_lossy() == "-") {
            codecs::to_stdout(
                &node,
                Some(
                    encode_options
                        .build(input.as_deref(), None, Format::Json, strip_options.clone())
                        .with_tool(tool, tool_args),
                ),
            )
            .await?;
        } else {
            for output in outputs {
                let strip_options = strip_options.clone();
                let tool = tool.clone();
                let tool_args = tool_args.clone();

                if output == PathBuf::from("-") {
                    codecs::to_stdout(
                        &node,
                        Some(
                            encode_options
                                .build(input.as_deref(), None, Format::Json, strip_options)
                                .with_tool(tool, tool_args),
                        ),
                    )
                    .await?;
                } else {
                    codecs::to_path(
                        &node,
                        &output,
                        Some(
                            encode_options
                                .build(input.as_deref(), Some(&output), Format::Json, strip_options)
                                .with_tool(tool, tool_args),
                        ),
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }
}
