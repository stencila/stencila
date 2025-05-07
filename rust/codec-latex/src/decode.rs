use codec::{
    common::{
        eyre::Result,
        once_cell::sync::Lazy,
        regex::{Captures, Regex},
        tracing,
    },
    format::Format,
    schema::{
        Article, Block, CodeChunk, CodeExpression, ForBlock, IfBlock, IfBlockClause, IncludeBlock,
        Inline, InlinesBlock, Node, RawBlock, Section, SectionType,
    },
    DecodeInfo, DecodeOptions,
};
use codec_pandoc::{pandoc_from_format, root_from_pandoc};

use crate::PANDOC_FORMAT;

/// Decode LaTeX
pub(super) async fn decode(
    latex: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let options = options.unwrap_or_default();

    // Default to coarse decoding to avoid loss of LaTeX not recognized by Pandoc
    if options.coarse.unwrap_or(true) {
        coarse(latex)
    } else {
        fine(latex, options).await
    }
}

/// Regex for custom commands and environments
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?sx)

        \\expr\{(?P<expr>[^}]*)\}

      |  \\input\{(?P<input>[^}]*)\}\n?

      | \\begin\{chunk\} \s*
          (?:\[(?P<chunk_opts>[^\]]*?)\])? \s* 
          (?P<chunk>.*?)
        \\end\{chunk\}\n?

      | \\begin\{for\}\{(?P<for_var>[\w_]+)\}\{(?P<for_code>[^\}]+)\} \s*
          (?P<for>.*?)
        \\end\{for\}\n?

      | \\begin\{if\}\{(?P<if_code>[^\}]+)\} \s*
          (?P<if>.*?)
        \\end\{if\}\n?

      | \\begin\{island\}
          (?P<island>.*?)
        \\end\{island\}\n?
    ",
    )
    .expect("invalid regex")
});

/// Decode LaTeX with the `--fine` option
///
/// Transforms custom LaTeX commands and environments into those recognized by
/// Pandoc. See the `pandoc-codec/src/blocks.rs` for why we encode things as we do below.
pub(super) async fn fine(latex: &str, options: DecodeOptions) -> Result<(Node, DecodeInfo)> {
    fn transform(latex: &str) -> String {
        RE.replace_all(latex, |captures: &Captures| {
            if let Some(mat) = captures.name("expr") {
                // Transform to lstinline expression
                ["\\lstinline[language=exec]{", mat.as_str(), "}"].concat()
            } else if let Some(mat) = captures.name("input") {
                // Transform \input to an "" environment environment with source as the content
                // because pandoc does not allow for args on unknown environments.
                // If we do not do this then Pandoc will attempt to do the transclusion itself
                let mut source = mat.as_str().to_string();
                if !source.ends_with(".tex") {
                    source.push_str(".tex");
                }
                ["\\begin{include}", &source, "\\end{include}"].concat()
            } else if let Some(mat) = captures.name("chunk") {
                // Transform to lstlisting environment
                [
                    "\\begin{lstlisting}[exec]\n",
                    mat.as_str(),
                    "\\end{lstlisting}\n",
                ]
                .concat()
            } else if let Some(mat) = captures.name("for") {
                // Passthrough as is but with content also transformed
                let variable = captures
                    .name("for_var")
                    .map(|var| var.as_str())
                    .unwrap_or_default();

                let code = captures
                    .name("for_code")
                    .map(|code| code.as_str())
                    .unwrap_or_default();

                [
                    "\\begin{for}{",
                    variable,
                    "}{",
                    code,
                    "}\n",
                    &transform(mat.as_str()),
                    "\\end{for}\n",
                ]
                .concat()
            } else if let Some(mat) = captures.name("if") {
                // Passthrough as is but with content also transformed
                let code = captures
                    .name("if_code")
                    .map(|code| code.as_str())
                    .unwrap_or_default();

                [
                    "\\begin{if}{",
                    code,
                    "}\n",
                    &transform(mat.as_str()),
                    "\\end{if}\n",
                ]
                .concat()
            } else if let Some(mat) = captures.name("island") {
                // No transformation required, parsed by Pandoc into a Div with class "island"
                [
                    "\\begin{section-island}\n",
                    &transform(mat.as_str()),
                    "\\end{section-island}\n",
                ]
                .concat()
            } else {
                tracing::error!("Unreachable branch reached");
                String::from("")
            }
        })
        .to_string()
    }

    let latex = transform(latex);
    let pandoc = pandoc_from_format(&latex, None, PANDOC_FORMAT, options.tool_args).await?;
    root_from_pandoc(pandoc, Format::Latex)
}

/// Decode LaTeX with the `--course` option
///
/// Decodes into an [`Article`] with only [`RawBlock`]s and executable block types
pub(super) fn coarse(latex: &str) -> Result<(Node, DecodeInfo)> {
    Ok((
        Node::Article(Article::new(latex_to_blocks(latex))),
        DecodeInfo::none(),
    ))
}

/// Decode LaTeX into a vector of "coarse" [`Block`]s
fn latex_to_blocks(latex: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut cursor = 0;

    for captures in RE.captures_iter(latex) {
        let mat = captures.get(0).expect("always present");
        if mat.start() > cursor {
            blocks.push(Block::RawBlock(RawBlock::new(
                Format::Latex.to_string(),
                latex[cursor..mat.start()].into(),
            )));
        }

        if let Some(mat) = captures.name("expr") {
            let code = mat.as_str().into();

            blocks.push(Block::InlinesBlock(InlinesBlock::new(vec![
                Inline::CodeExpression(CodeExpression {
                    code,
                    ..Default::default()
                }),
            ])));
        } else if let Some(mat) = captures.name("input") {
            let mut source = mat.as_str().to_string();
            if !source.ends_with(".tex") {
                source.push_str(".tex");
            }

            blocks.push(Block::IncludeBlock(IncludeBlock::new(source)));
        } else if let Some(mat) = captures.name("chunk") {
            let code = mat.as_str().into();

            let mut programming_language = None;
            let mut is_echoed = None;
            let mut is_hidden = None;
            if let Some(options) = captures.name("chunk_opts") {
                for option in options
                    .as_str()
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    if option == "hide" {
                        is_hidden = Some(true);
                    } else if option == "echo" {
                        is_echoed = Some(true);
                    } else if programming_language.is_none() {
                        programming_language = Some(option.to_string());
                    }
                }
            }

            blocks.push(Block::CodeChunk(CodeChunk {
                programming_language,
                is_hidden,
                is_echoed,
                code,
                ..Default::default()
            }));
        } else if let Some(mat) = captures.name("for") {
            let variable = captures
                .name("for_var")
                .map(|var| var.as_str())
                .unwrap_or_default()
                .into();

            let code = captures
                .name("for_code")
                .map(|code| code.as_str())
                .unwrap_or_default()
                .into();

            blocks.push(Block::ForBlock(ForBlock {
                variable,
                code,
                content: latex_to_blocks(mat.as_str()),
                ..Default::default()
            }));
        } else if let Some(mat) = captures.name("if") {
            let code = captures
                .name("if_code")
                .map(|code| code.as_str())
                .unwrap_or_default()
                .into();

            blocks.push(Block::IfBlock(IfBlock::new(vec![IfBlockClause {
                code,
                content: latex_to_blocks(mat.as_str()),
                ..Default::default()
            }])));
        } else if let Some(mat) = captures.name("island") {
            blocks.push(Block::Section(Section {
                section_type: Some(SectionType::Island),
                content: latex_to_blocks(mat.as_str()),
                ..Default::default()
            }));
        }

        cursor = mat.end();
    }

    if cursor < latex.len() {
        blocks.push(Block::RawBlock(RawBlock::new(
            Format::Latex.to_string(),
            latex[cursor..].into(),
        )));
    }

    blocks
}
