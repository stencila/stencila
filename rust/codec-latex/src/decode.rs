use std::sync::LazyLock;

use itertools::Itertools;
use regex::{Captures, Regex};

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{
        AppendixBreak, Article, Block, CodeChunk, CodeExpression, ForBlock, Heading, IfBlock,
        IfBlockClause, IncludeBlock, Inline, InlinesBlock, Island, LabelType, Link, Node, RawBlock,
        Text,
    },
};
use stencila_codec_pandoc::{pandoc_from_format, root_from_pandoc};

/// Decode LaTeX
pub(super) async fn decode(
    latex: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    // Default to coarse decoding to avoid loss of LaTeX not recognized by Pandoc
    if options
        .as_ref()
        .and_then(|opts| opts.coarse)
        .unwrap_or(true)
    {
        coarse(latex, options)
    } else {
        fine(latex, options).await
    }
}

/// Regex for custom commands and environments
static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?sx)

        \\expr\{(?P<expr>[^}]*)\}

      | \\(auto)?ref\{(?P<ref>[^}]*)\}

      | \\input\{(?P<input>[^}]*)\}\s*\n?

      | (?P<appendix>\\appendix)\s*\n?

      | (?:\\label\{(?P<label_before>[^}]*)\}\s*)?\\section\{(?P<section>[^}]*)\}\s*(?:\\label\{(?P<label_after>[^}]*)\}\s*)?\n?

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

      | \\begin\{island\} \s*
          (?:\[(?P<island_opts>[^\]]*?)\])? \s* 
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
pub(super) async fn fine(
    latex: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    fn transform(latex: &str) -> String {
        RE.replace_all(latex, |captures: &Captures| {
            if let Some(mat) = captures.name("expr") {
                // Transform to lstinline expression
                ["\\lstinline[language=exec]{", mat.as_str(), "}"].concat()
            } else if let Some(mat) = captures.name("input") {
                // Transform \input to an include environment with source as the content
                // because pandoc does not allow for args on unknown environments.
                // If we do not do this then Pandoc will attempt to do the transclusion itself
                let mut source = mat.as_str().to_string();
                if !source.ends_with(".tex") {
                    source.push_str(".tex");
                }
                ["\\begin{include}", &source, "\\end{include}"].concat()
            } else if captures.name("appendix").is_some() {
                // Pandoc will consume \appendix and not produce a node so
                // to preserve this, create an empty custom env
                ["\\begin{appendix}\\end{appendix}"].concat()
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
                    "\\begin{island}\n",
                    &transform(mat.as_str()),
                    "\\end{island}\n",
                ]
                .concat()
            } else {
                // Pass through things that do not need to be transformed (e.g. ref, autoref)
                captures[0].to_string()
            }
        })
        .to_string()
    }

    let latex = transform(latex);
    let pandoc = pandoc_from_format(&latex, None, "latex", &options).await?;
    root_from_pandoc(pandoc, Format::Latex, &options)
}

/// Decode LaTeX with the `--course` option
///
/// Decodes into an [`Article`] with only [`RawBlock`]s and executable block types
pub(super) fn coarse(latex: &str, options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    let options = options.unwrap_or_default();

    let latex = if !options.island_wrap.is_empty() {
        wrap_island_envs(latex, &options.island_wrap, &options.island_style)?
    } else {
        latex.into()
    };

    Ok((
        Node::Article(Article::new(latex_to_blocks(&latex, &options.island_style))),
        DecodeInfo::none(),
    ))
}

/// Wrap specified environments in
fn wrap_island_envs(
    input: &str,
    island_envs: &[String],
    island_style: &Option<String>,
) -> Result<String> {
    let style = island_style
        .as_ref()
        .map(|style| format!("style={style}"))
        .unwrap_or_default();

    let mut output = input.to_owned();
    for env in island_envs {
        let re = Regex::new(&format!(r"(?s)\\begin\{{{env}\}}.*?\\end\{{{env}\}}"))?;

        output = re
            .replace_all(&output, |captures: &Captures| {
                format!(
                    "\\begin{{island}}[auto,{}]{}\\end{{island}}",
                    style, &captures[0]
                )
            })
            .into_owned();
    }

    Ok(output)
}

/// Decode LaTeX into a vector of "coarse" [`Block`]s
fn latex_to_blocks(latex: &str, island_style: &Option<String>) -> Vec<Block> {
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
        } else if let Some(mat) = captures.name("ref") {
            let target = ["#", mat.as_str()].concat();
            let label_only = captures[0].contains("\\ref{").then_some(true);

            blocks.push(Block::InlinesBlock(InlinesBlock::new(vec![Inline::Link(
                Link {
                    target,
                    label_only,
                    ..Default::default()
                },
            )])));
        } else if let Some(mat) = captures.name("input") {
            let mut source = mat.as_str().to_string();

            // If source does not have an extension then assume tex
            if !source.contains(".") {
                source.push_str(".tex");
            }

            blocks.push(Block::IncludeBlock(IncludeBlock::new(source)));
        } else if captures.name("appendix").is_some() {
            blocks.push(Block::AppendixBreak(AppendixBreak::new()));
        } else if let Some(section) = captures.name("section") {
            let id = captures
                .name("label_before")
                .or(captures.name("label_after"))
                .map(|cap| cap.as_str().to_string());

            blocks.push(Block::Heading(Heading {
                id,
                level: 1,
                content: vec![Inline::Text(Text::from(section.as_str()))],
                ..Default::default()
            }));
        } else if let Some(mat) = captures.name("chunk") {
            let code = mat.as_str().to_string();

            let mut id = None;
            let mut programming_language = None;
            let mut is_echoed = None;
            let mut is_hidden = None;
            let mut code_options = Vec::new();
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
                    } else if !option.contains("=") && programming_language.is_none() {
                        programming_language = Some(option.to_string());
                    } else if let Some((name, value)) =
                        option.split("=").map(|s| s.trim()).collect_tuple()
                    {
                        if name == "id" {
                            id = Some(value.to_string())
                        } else if name == "hide" {
                            is_hidden = Some(value.to_lowercase() == "true")
                        } else if name == "echo" {
                            is_echoed = Some(value.to_lowercase() == "true")
                        } else {
                            code_options.push((name, value))
                        }
                    }
                }
            }

            let code = if code_options.is_empty() {
                code
            } else {
                let mut new_code = String::with_capacity(code.len() + code_options.len() * 20);
                for (name, value) in code_options {
                    new_code.push_str(&match programming_language.as_deref() {
                        Some("js" | "javascript") => ["// @", name, " ", value, "\n"].concat(),
                        // Use Knitr style attribute comments: https://quarto.org/docs/reference/cells/cells-knitr.html
                        _ => ["#| ", name, ": ", value, "\n"].concat(),
                    });
                }
                new_code.push_str(&code);
                new_code
            }
            .into();

            blocks.push(Block::CodeChunk(CodeChunk {
                id,
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
                content: latex_to_blocks(mat.as_str(), island_style),
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
                content: latex_to_blocks(mat.as_str(), island_style),
                ..Default::default()
            }])));
        } else if let Some(mat) = captures.name("island") {
            let mut id = None;
            let mut is_automatic = None;
            let mut label_type = None;
            let mut label = None;
            let mut label_automatically = None;
            let mut style = island_style.clone();
            if let Some(options) = captures.name("island_opts") {
                for option in options
                    .as_str()
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    if let Some((name, value)) = option.split("=").map(|s| s.trim()).collect_tuple()
                    {
                        if name == "label-type" {
                            label_type = match value.to_lowercase().as_str() {
                                "tab" | "table" => Some(LabelType::TableLabel),
                                "fig" | "figure" => Some(LabelType::FigureLabel),
                                _ => None,
                            }
                        } else if name == "label" {
                            label = Some(value.to_string());
                            label_automatically = Some(false);
                        } else if name == "style" {
                            style = Some(value.to_string())
                        } else if name == "id" {
                            id = Some(value.to_string())
                        }
                    } else if option == "auto" {
                        is_automatic = Some(true);
                    } else if id.is_none() {
                        id = Some(option.to_string())
                    }
                }
            }

            let content = mat.as_str();

            // If no id is specified, try to get from the \label command within the table
            if id.is_none() {
                static RE: LazyLock<Regex> =
                    LazyLock::new(|| Regex::new(r"\\label\{([^}]+)\}").expect("invalid regex"));
                if let Some(label) = RE
                    .captures(content)
                    .and_then(|caps| caps.get(1))
                    .map(|mat| mat.as_str())
                {
                    id = Some(label.to_string())
                }
            }

            if let (Some(id), None) = (&id, &label_type) {
                if id.starts_with("tab:") || id.starts_with("tbl-") {
                    label_type = Some(LabelType::TableLabel);
                } else if id.starts_with("fig:") || id.starts_with("fig-") {
                    label_type = Some(LabelType::FigureLabel);
                }
            }

            blocks.push(Block::Island(Island {
                id,
                is_automatic,
                label_type,
                label,
                label_automatically,
                style,
                content: latex_to_blocks(content, island_style),
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
