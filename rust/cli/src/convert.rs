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
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self {
            input,
            outputs,
            decode_options,
            encode_options,
            strip_options,
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
                Some(encode_options.build(
                    input.as_deref(),
                    None,
                    Format::Json,
                    strip_options.clone(),
                )),
            )
            .await?;
        } else {
            for output in outputs {
                if output == PathBuf::from("-") {
                    codecs::to_stdout(
                        &node,
                        Some(encode_options.build(
                            input.as_deref(),
                            None,
                            Format::Json,
                            strip_options.clone(),
                        )),
                    )
                    .await?;
                } else {
                    codecs::to_path(
                        &node,
                        &output,
                        Some(encode_options.build(
                            input.as_deref(),
                            Some(&output),
                            Format::Json,
                            strip_options.clone(),
                        )),
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }
}
