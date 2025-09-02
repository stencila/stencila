use std::path::PathBuf;

use clap::Parser;
use eyre::Result;

use cli_utils::color_print::cstr;
use format::Format;

use crate::options::{DecodeOptions, EncodeOptions, StripOptions};

/// Convert a document to another format
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path, URL or other identifier for the input file
    ///
    /// If not supplied, or if "-", the input content is read from `stdin`.
    input: Option<String>,

    /// The paths of desired output files
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

    /// The tool to use for decoding inputs
    ///
    /// Only supported for formats that use alternative external tools for
    /// decoding inputs and ignored otherwise. Use `--tool` for specifying the
    /// tool to use for encoding outputs.
    #[arg(long)]
    from_tool: Option<String>,

    /// The tool to use for encoding outputs (e.g. pandoc)
    ///
    /// Only supported for formats that use alternative external tools for encoding and ignored otherwise.
    /// Use `--from-tool` for specifying the tool to use for decoding inputs.
    #[arg(long, alias = "to-tool")]
    tool: Option<String>,

    /// Arguments to pass through to the tool using for encoding
    ///
    /// Only supported for formats that use external tools for encoding and ignored otherwise.
    /// Note: these arguments are not used for decoding from the input, only for encoding to the output.
    #[arg(last = true, allow_hyphen_values = true)]
    tool_args: Vec<String>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Convert Stencila Markdown to MyST Markdown</dim>
  <b>stencila convert</> <g>document.smd</> <g>document.myst</>

  <dim># Convert to multiple output formats</dim>
  <b>stencila convert</> <g>input.smd</> <g>output.html</> <g>output.pdf</> <g>output.docx</>

  <dim># Specify input and output formats explicitly</dim>
  <b>stencila convert</> <g>input.txt</> <g>output.json</> <c>--from</> <g>plain</> <c>--to</> <g>json</>

  <dim># Convert with specific codec options</dim>
  <b>stencila convert</> <g>doc.md</> <g>doc.html</> <c>--standalone</>

  <dim># Convert only specific pages from a PDF</dim>
  <b>stencila convert</> <g>document.pdf</> <g>extract.md</> <c>--pages</> <g>1,3,5-10</>

  <dim># Convert all pages except specific ones</dim>
  <b>stencila convert</> <g>report.pdf</> <g>content.md</> <c>--exclude-pages</> <g>5,15</>

  <dim># Convert only odd pages from a document</dim>
  <b>stencila convert</> <g>book.pdf</> <g>odd-pages.md</> <c>--pages</> <g>odd</>

  <dim># Use an external tool like Pandoc</dim>
  <b>stencila convert</> <g>doc.md</> <g>doc.tex</> <c>--tool</> <g>pandoc</>

  <dim># Pass arguments to external tool</dim>
  <b>stencila convert</> <g>doc.md</> <g>doc.pdf</> <c>--tool</> <g>pandoc</> <c>--</> <c>--pdf-engine=</><g>xelatex</>

  <dim># Convert from stdin to stdout (defaults to JSON)</dim>
  <y>echo \"# Hello\"</> <b>|</> <b>stencila convert</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Self {
            input,
            outputs,
            decode_options,
            encode_options,
            strip_options,
            from_tool,
            tool,
            tool_args,
        } = self;

        let input_path = input
            .as_ref()
            .map(PathBuf::from)
            .and_then(|path| path.exists().then_some(path));

        let input = input.as_deref().unwrap_or("-");

        let decode_options = decode_options
            .build(input_path.as_deref(), strip_options.clone())
            .with_tool(from_tool, Vec::new());
        let node = codecs::from_identifier(input, Some(decode_options)).await?;

        if outputs.is_empty() || outputs.iter().all(|path| path.to_string_lossy() == "-") {
            codecs::to_stdout(
                &node,
                Some(
                    encode_options
                        .build(
                            input_path.as_deref(),
                            None,
                            Format::Json,
                            strip_options.clone(),
                        )
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
                                .build(input_path.as_deref(), None, Format::Json, strip_options)
                                .with_tool(tool, tool_args),
                        ),
                    )
                    .await?;
                } else {
                    let completed = codecs::to_path(
                        &node,
                        &output,
                        Some(
                            encode_options
                                .build(
                                    input_path.as_deref(),
                                    Some(&output),
                                    Format::Json,
                                    strip_options,
                                )
                                .with_tool(tool, tool_args),
                        ),
                    )
                    .await?;

                    #[allow(clippy::print_stderr)]
                    if completed {
                        eprintln!(
                            "📑 Successfully converted `{input}` to `{}`",
                            output.display()
                        )
                    } else {
                        eprintln!("⏭️  Skipped converting `{input}`")
                    }
                }
            }
        }

        Ok(())
    }
}
