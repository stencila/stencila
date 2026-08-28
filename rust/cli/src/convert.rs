use std::{
    borrow::Cow,
    env::current_dir,
    path::{Path, PathBuf},
};

use clap::Parser;
use eyre::Result;

use stencila_cli_utils::color_print::cstr;
use stencila_format::Format;
use stencila_node_supplements::{embed_supplements, extract_supplements};

use stencila_node_suggestions::{ResolveSuggestions, SuggestionAction, review::interactive_review};
use stencila_schema::Node;
use url::Url;

use crate::options::{DecodeOptions, EncodeOptions, StripOptions, SuggestionOptions};

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
    strip_options: StripOptions, // Place here because decode -> structuring -> stripping -> encoding

    #[command(flatten)]
    suggestion_options: SuggestionOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

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
            suggestion_options,
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
        let mut node = stencila_codecs::from_identifier(input, Some(decode_options)).await?;

        // Resolve suggestions if requested
        if suggestion_options.accept_suggestions {
            node.resolve_suggestions(&SuggestionAction::AcceptAll);
        } else if suggestion_options.reject_suggestions {
            node.resolve_suggestions(&SuggestionAction::RejectAll);
        } else if suggestion_options.review_suggestions {
            let action = interactive_review(&node).await?;
            node.resolve_suggestions(&action);
        }

        if outputs.is_empty() || outputs.iter().all(|path| path.to_string_lossy() == "-") {
            let options = encode_options
                .build(
                    input_path.as_deref(),
                    None,
                    Format::Json,
                    strip_options.clone(),
                )
                .with_tool(tool, tool_args);
            let node = node_for_encoding(&node, &options, input_path.as_deref())?;
            stencila_codecs::to_stdout(node.as_ref(), Some(options)).await?;
        } else {
            for output in outputs {
                let strip_options = strip_options.clone();
                let tool = tool.clone();
                let tool_args = tool_args.clone();

                if output == std::path::Path::new("-") {
                    let options = encode_options
                        .build(input_path.as_deref(), None, Format::Json, strip_options)
                        .with_tool(tool, tool_args);
                    let node = node_for_encoding(&node, &options, input_path.as_deref())?;
                    stencila_codecs::to_stdout(node.as_ref(), Some(options)).await?;
                } else {
                    let encode_options = encode_options
                        .build(
                            input_path.as_deref(),
                            Some(&output),
                            Format::Json,
                            strip_options,
                        )
                        .with_tool(tool, tool_args);

                    if let Some(dir) = encode_options.extract_supplements.as_ref() {
                        extract_supplements(&mut node, &output, dir).await?;
                    } else if encode_options.embed_supplements.unwrap_or_default() {
                        let input_path = match input_path.as_ref() {
                            Some(path) => path,
                            None => &current_dir()?,
                        };
                        embed_supplements(&mut node, input_path).await?;
                    }

                    let node = node_for_encoding(&node, &encode_options, input_path.as_deref())?;
                    let completed =
                        stencila_codecs::to_path(node.as_ref(), &output, Some(encode_options))
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

fn node_for_encoding<'a>(
    node: &'a Node,
    options: &stencila_codecs::EncodeOptions,
    input_path: Option<&Path>,
) -> Result<Cow<'a, Node>> {
    if options.format.as_ref() != Some(&Format::MiraJsonLd) || matches!(node, Node::Graph(..)) {
        return Ok(Cow::Borrowed(node));
    }

    let subject = input_path
        .and_then(|path| path.canonicalize().ok())
        .and_then(|path| Url::from_file_path(path).ok())
        .map(|url| url.to_string())
        .unwrap_or_else(|| "mira:document".to_string());
    let graph = stencila_graph::graph_from_node(subject, node)?;
    Ok(Cow::Owned(Node::Graph(graph)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_schema::{Article, Block, Claim};

    #[test]
    fn prepares_documents_for_mira_encoding() -> Result<()> {
        let mut claim = Claim::new(Vec::new());
        claim.id = Some("claim-1".to_string());
        let node = Node::Article(Article::new(vec![Block::Claim(claim)]));
        let options = stencila_codecs::EncodeOptions {
            format: Some(Format::MiraJsonLd),
            ..Default::default()
        };

        let prepared = node_for_encoding(&node, &options, None)?;
        let Node::Graph(graph) = prepared.as_ref() else {
            eyre::bail!("expected MIRA preparation to produce a graph")
        };
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| matches!(node.node.as_ref(), Node::Claim(..)))
        );
        Ok(())
    }
}
