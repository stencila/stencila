use std::path::PathBuf;

use cli_utils::{Code, ToStdout};
use common::{
    clap::{self, Parser},
    eyre::Result,
};
use format::Format;

use crate::options::{DecodeOptions, EncodeOptions, StripOptions};
use color_print::cstr;

pub static AFTER_HELP: &str = cstr!("
<bold>Usage Instructions</bold>

<bold>Example:</bold> <cyan>`stencila convert article.smd article.docx`</cyan>

Detailed usage information provided in long-form help page,
available by <cyan>`stencila new --help`</cyan>
");

pub static AFTER_LONG_HELP: &str = cstr!("
<bold> Further Information and Examples </bold>

    <bold>Examples:</bold><cyan>
        `stencila convert article.md --to json --pretty`
        `stencila convert article.md --to jats --compact`
        `stencila convert article.docx article.smd --input-losses --output-losses`
        `stencila convert article.md --to docx article.docx`
        `stencila convert article.smd  article.docx --render`
    </cyan>

    The File extension must be one of the supported codecs which can be found using <cyan>`stencila codecs list`</cyan>
");

/// Convert a document to another format
///
/// Examples:
///
/// `stecila convert article.smd article.docx`
///
/// `stecila convert file.txt --from markdown --to json --pretty`
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

    /// The format to encode from (or codec to use)
    ///
    /// Defaults to inferring the format from the file name extension
    /// of the `input`.
    /// See `stencila codecs list` for available formats.
    #[arg(long, short)]
    from: Option<String>,

    /// The format to encode to (or codec to use)
    ///
    /// Defaults to inferring the format from the file name extension
    /// of the `output`. If no `output` is supplied, defaults to JSON.
    /// See `stencila codecs list` for available formats.
    #[arg(long, short)]
    to: Option<String>,

    /// What to do if there are losses when decoding from the input
    ///
    /// Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or
    /// a filename to write the losses to (only `json` or `yaml` file extensions are supported).
    #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
    input_losses: codecs::LossesResponse,

    /// What to do if there are losses when encoding to the output
    ///
    /// See help for `--input-losses` for details.
    #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
    output_losses: codecs::LossesResponse,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    #[command(flatten)]
    strip_options: StripOptions,

    /// Arguments to pass through to any CLI tool delegated to for conversion (e.g. Pandoc)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    passthrough_args: Vec<String>,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self {
            input,
            output,
            from,
            to,
            input_losses,
            output_losses,
            decode_options,
            encode_options,
            strip_options,
            passthrough_args,
        } = self;

        let decode_options = decode_options.build(
            from,
            strip_options.clone(),
            input_losses,
            passthrough_args.clone(),
        );
        let encode_options = encode_options.build(
            input.as_deref(),
            output.as_deref(),
            to,
            Format::Json,
            strip_options,
            output_losses,
            passthrough_args.clone(),
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
