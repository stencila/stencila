use std::{str::FromStr, sync::LazyLock};

use itertools::Itertools;
use regex::{Captures, Regex};

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{
        AppendixBreak, Article, Author, Bibliography, Block, Citation, CitationGroup, CitationMode,
        CitationOptions, CodeChunk, CodeExpression, CompilationMessage, Date, ForBlock, Heading,
        IfBlock, IfBlockClause, IncludeBlock, Inline, InlinesBlock, Island, LabelType, Link,
        MessageLevel, Node, Paragraph, Person, RawBlock, Section, SectionType, Text,
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
static COMMANDS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?sx)

        \\expr\{(?P<expr>[^}]*)\}

      | \\(auto)?ref\{(?P<ref>[^}]*)\}

      | \\(?P<cite_cmd>citep?|citet|citeauthor|citeyear|citealt|citealp)
          (?:\[(?P<cite_arg1>[^\]]*)\])?
          (?:\[(?P<cite_arg2>[^\]]*)\])?
          \{(?P<cite_keys>[^}]+)\}

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

      | (?P<printbib>\\printbibliography)(?:\[[^\]]*\])?\s*\n?
    ",
    )
    .expect("invalid regex")
});

/// Regex for bibliography commands
static BIBLIOGRAPHY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\\(?:bibliography|addbibresource)\{([^}]+)\}").expect("invalid regex")
});

/// Regex for \title{...} command
static TITLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\title\{([^}]*)\}").expect("invalid regex"));

/// Regex for \author{...} command
/// Note: Authors can be separated by \and or \\ within the braces
static AUTHOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\author\{([^}]*(?:\{[^}]*\}[^}]*)*)\}").expect("invalid regex"));

/// Regex for \date{...} command
static DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\date\{([^}]*)\}").expect("invalid regex"));

/// Regex for \keywords{...} command (used by some document classes)
static KEYWORDS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\keywords\{([^}]*)\}").expect("invalid regex"));

/// Regex for \begin{abstract}...\end{abstract} environment
static ABSTRACT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)\\begin\{abstract\}(.*?)\\end\{abstract\}").expect("invalid regex")
});

/// Extract bibliography source from LaTeX
///
/// Returns the bibliography source path and any compilation messages
fn extract_bibliography(latex: &str) -> (Option<Bibliography>, Vec<CompilationMessage>) {
    let mut messages = Vec::new();
    let matches: Vec<_> = BIBLIOGRAPHY_RE.captures_iter(latex).collect();

    if matches.is_empty() {
        return (None, messages);
    }

    // Warn if multiple bibliography commands found
    if matches.len() > 1 {
        messages.push(CompilationMessage::new(
            MessageLevel::Warning,
            format!(
                "Multiple bibliography commands found ({}), using first",
                matches.len()
            ),
        ));
    }

    // Get the first match
    let source = matches[0]
        .get(1)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();

    // Ensure .bib extension
    let source = if source.ends_with(".bib") {
        source
    } else {
        format!("{}.bib", source)
    };

    (
        Some(Bibliography {
            source,
            ..Default::default()
        }),
        messages,
    )
}

/// Extract title from LaTeX \title{...} command
fn extract_title(latex: &str) -> Option<Vec<Inline>> {
    TITLE_RE.captures(latex).and_then(|caps| {
        caps.get(1).map(|m| {
            let title_text = m.as_str().trim();
            if title_text.is_empty() {
                vec![]
            } else {
                vec![Inline::Text(Text::from(title_text))]
            }
        })
    })
}

/// Extract authors from LaTeX \author{...} command
///
/// Authors can be separated by \and or \\ (LaTeX conventions)
fn extract_authors(latex: &str) -> Option<Vec<Author>> {
    /// Remove common LaTeX commands from a string
    fn remove_latex_commands(s: &str) -> String {
        // Remove \thanks{...}, \footnote{...}, etc.
        static CMD_RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"\\(?:thanks|footnote|textsuperscript)\{[^}]*\}").expect("invalid regex")
        });

        CMD_RE.replace_all(s, "").into_owned()
    }

    AUTHOR_RE.captures(latex).and_then(|caps| {
        caps.get(1).map(|m| {
            let authors_str = m.as_str().trim();
            if authors_str.is_empty() {
                return vec![];
            }

            // Split by \and or \\ to separate multiple authors
            let author_names: Vec<&str> = authors_str
                .split(r"\and")
                .flat_map(|s| s.split(r"\\"))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            author_names
                .into_iter()
                .filter_map(|name| {
                    // Remove any LaTeX commands that might be in the name (like \thanks{...})
                    let clean_name = remove_latex_commands(name);
                    let clean_name = clean_name.trim();
                    if clean_name.is_empty() {
                        return None;
                    }

                    // Try to parse name into given and family names
                    Person::from_str(name).ok().map(Author::Person)
                })
                .collect()
        })
    })
}

/// Extract date from LaTeX \date{...} command
fn extract_date(latex: &str) -> Option<Date> {
    DATE_RE.captures(latex).and_then(|caps| {
        caps.get(1).and_then(|m| {
            let date_str = m.as_str().trim();

            // Skip empty dates or \today
            if date_str.is_empty() || date_str == r"\today" {
                return None;
            }

            // Try to normalize the date to ISO 8601 format
            // For now, just store as-is if it's not empty
            Some(Date::new(date_str.into()))
        })
    })
}

/// Extract keywords from LaTeX \keywords{...} command
fn extract_keywords(latex: &str) -> Option<Vec<String>> {
    KEYWORDS_RE.captures(latex).and_then(|caps| {
        caps.get(1).map(|m| {
            let keywords_str = m.as_str().trim();
            if keywords_str.is_empty() {
                return vec![];
            }

            // Keywords are typically comma-separated or semicolon-separated
            keywords_str
                .split([',', ';'])
                .map(|s| s.trim().into())
                .filter(|s: &String| !s.is_empty())
                .collect()
        })
    })
}

/// Extract abstract from LaTeX \begin{abstract}...\end{abstract} environment
fn extract_abstract(latex: &str) -> Option<Vec<Block>> {
    ABSTRACT_RE.captures(latex).and_then(|caps| {
        caps.get(1).map(|m| {
            let abstract_text = m.as_str().trim();
            if abstract_text.is_empty() {
                vec![]
            } else {
                // Create a paragraph with the abstract text
                vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                    Text::from(abstract_text),
                )]))]
            }
        })
    })
}

/// Decode LaTeX with the `--fine` option
///
/// Transforms custom LaTeX commands and environments into those recognized by
/// Pandoc. See the `pandoc-codec/src/blocks.rs` for why we encode things as we do below.
pub(super) async fn fine(
    latex: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    fn transform(latex: &str) -> String {
        COMMANDS_RE
            .replace_all(latex, |captures: &Captures| {
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

    // Extract metadata from the original LaTeX before transformation
    let authors = extract_authors(latex);
    let keywords = extract_keywords(latex);
    let r#abstract = extract_abstract(latex);

    let latex = transform(latex);
    let pandoc = pandoc_from_format(&latex, None, "latex", &options).await?;
    let (node, info) = root_from_pandoc(pandoc, Format::Latex, &options)?;

    // Apply extracted metadata to the article if not already set by Pandoc
    let node = if let Node::Article(mut article) = node {
        // Only set authors if not already set by Pandoc
        if article.authors.is_none()
            && let Some(authors) = authors
            && !authors.is_empty()
        {
            article.authors = Some(authors);
        }

        // Only set keywords if not already set
        if article.options.keywords.is_none()
            && let Some(keywords) = keywords
            && !keywords.is_empty()
        {
            article.options.keywords = Some(keywords);
        }

        // Only set abstract if not already set or empty
        // Pandoc may create an abstract with empty paragraphs, so check for that
        if (article.r#abstract.is_none()
            || article.r#abstract.as_ref().is_some_and(|a| {
                a.is_empty()
                    || a.iter()
                        .all(|b| matches!(b, Block::Paragraph(p) if p.content.is_empty()))
            }))
            && let Some(r#abstract) = r#abstract
            && !r#abstract.is_empty()
        {
            article.r#abstract = Some(r#abstract);
        }

        Node::Article(article)
    } else {
        node
    };

    Ok((node, info))
}

/// Decode LaTeX with the `--coarse` option
///
/// Decodes into an [`Article`] with only [`RawBlock`]s and executable block types
pub(super) fn coarse(latex: &str, options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    let options = options.unwrap_or_default();

    let latex = if !options.island_wrap.is_empty() {
        wrap_island_envs(latex, &options.island_wrap, &options.island_style)?
    } else {
        latex.into()
    };

    // Extract metadata from the LaTeX source
    let (bibliography, bib_messages) = extract_bibliography(&latex);
    let title = extract_title(&latex);
    let authors = extract_authors(&latex);
    let date_published = extract_date(&latex);
    let keywords = extract_keywords(&latex);
    let r#abstract = extract_abstract(&latex);

    // Create article with content
    let mut article = Article::new(latex_to_blocks(&latex, &options.island_style));

    // Set metadata fields
    if let Some(title) = title
        && !title.is_empty()
    {
        article.title = Some(title);
    }

    if let Some(authors) = authors
        && !authors.is_empty()
    {
        article.authors = Some(authors);
    }

    if date_published.is_some() {
        article.date_published = date_published;
    }

    if let Some(keywords) = keywords
        && !keywords.is_empty()
    {
        article.options.keywords = Some(keywords);
    }

    if let Some(r#abstract) = r#abstract
        && !r#abstract.is_empty()
    {
        article.r#abstract = Some(r#abstract);
    }

    // Set bibliography if found
    if bibliography.is_some() {
        article.options.bibliography = bibliography;
    }

    // Set compilation messages if any
    if !bib_messages.is_empty() {
        article.options.compilation_messages = Some(bib_messages);
    }

    Ok((Node::Article(article), DecodeInfo::none()))
}

/// Wrap specified environments in island tags
///
/// Only wraps environments in non-comment segments to avoid wrapping
/// commented-out code.
fn wrap_island_envs(
    input: &str,
    island_envs: &[String],
    island_style: &Option<String>,
) -> Result<String> {
    let style = island_style
        .as_ref()
        .map(|style| format!("style={style}"))
        .unwrap_or_default();

    // Process each segment separately, only wrapping in non-comment segments
    let mut output = String::new();
    for (is_comment, segment) in segment_by_comments(input) {
        if is_comment {
            // Pass through comment segments unchanged
            output.push_str(&segment);
        } else {
            // Apply island wrapping only to non-comment segments
            let mut wrapped = segment;
            for env in island_envs {
                let re = Regex::new(&format!(r"(?s)\\begin\{{{env}\}}.*?\\end\{{{env}\}}"))?;
                wrapped = re
                    .replace_all(&wrapped, |captures: &Captures| {
                        format!(
                            "\\begin{{island}}[auto,{}]{}\\end{{island}}",
                            style, &captures[0]
                        )
                    })
                    .into_owned();
            }
            output.push_str(&wrapped);
        }
    }

    Ok(output)
}

/// Segment LaTeX into comment and non-comment regions
///
/// Returns a vector of (is_comment, content) tuples where adjacent comment
/// lines are grouped together. Blank lines continue the previous segment type
/// to avoid losing whitespace between segments.
fn segment_by_comments(latex: &str) -> Vec<(bool, String)> {
    let mut segments = Vec::new();
    let mut current_is_comment = false;
    let mut current_content = String::new();
    let mut started = false;

    for line in latex.lines() {
        let trimmed = line.trim_start();
        // Blank lines continue the current segment type (don't trigger a switch)
        let is_comment = if trimmed.is_empty() {
            current_is_comment
        } else {
            trimmed.starts_with('%')
        };

        if !started {
            current_is_comment = is_comment;
            started = true;
        }

        if is_comment == current_is_comment {
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
        } else {
            if !current_content.is_empty() {
                segments.push((current_is_comment, current_content));
            }
            current_content = line.to_string();
            current_is_comment = is_comment;
        }
    }

    if !current_content.is_empty() {
        segments.push((current_is_comment, current_content));
    }

    // Add trailing newline to all segments except the last (to separate them when joined)
    // The last segment only gets a newline if the original input had one
    let len = segments.len();
    for (i, (_, content)) in segments.iter_mut().enumerate() {
        if i < len - 1 {
            // Not the last segment - always add newline
            content.push('\n');
        } else if latex.ends_with('\n') {
            // Last segment - only add if original had trailing newline
            content.push('\n');
        }
    }

    segments
}

/// Decode LaTeX into a vector of "coarse" [`Block`]s
fn latex_to_blocks(latex: &str, island_style: &Option<String>) -> Vec<Block> {
    let mut blocks = Vec::new();

    for (is_comment, segment) in segment_by_comments(latex) {
        if is_comment {
            // Comment lines go directly into a RawBlock without regex matching
            blocks.push(Block::RawBlock(RawBlock::new(
                Format::Latex.to_string(),
                segment.into(),
            )));
        } else {
            // Non-comment content is processed with regex matching
            blocks.extend(process_latex_segment(&segment, island_style));
        }
    }

    blocks
}

/// Process a non-comment LaTeX segment into blocks
fn process_latex_segment(latex: &str, island_style: &Option<String>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut cursor = 0;

    for captures in COMMANDS_RE.captures_iter(latex) {
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
        } else if let Some(keys_match) = captures.name("cite_keys") {
            // Determine citation mode from command
            let citation_mode = match captures.name("cite_cmd").map(|m| m.as_str()) {
                Some("citet") => Some(CitationMode::Narrative),
                Some("citeauthor") => Some(CitationMode::NarrativeAuthor),
                Some("citeyear") => Some(CitationMode::NarrativeYear),
                _ => Some(CitationMode::Parenthetical),
            };

            // Parse prefix/suffix - natbib uses [prefix][suffix] or just [suffix]
            let (prefix, suffix) = match (captures.name("cite_arg1"), captures.name("cite_arg2")) {
                (Some(p), Some(s)) => (
                    Some(p.as_str().to_string()).filter(|s| !s.is_empty()),
                    Some(s.as_str().to_string()).filter(|s| !s.is_empty()),
                ),
                (Some(s), None) => (None, Some(s.as_str().to_string()).filter(|s| !s.is_empty())),
                _ => (None, None),
            };

            // Parse comma-separated keys
            let citations: Vec<Citation> = keys_match
                .as_str()
                .split(',')
                .map(|key| key.trim())
                .filter(|key| !key.is_empty())
                .map(|key| Citation {
                    target: key.to_string(),
                    citation_mode,
                    options: Box::new(CitationOptions {
                        citation_prefix: prefix.clone(),
                        citation_suffix: suffix.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .collect();

            if !citations.is_empty() {
                let inline = if citations.len() == 1 {
                    Inline::Citation(citations.into_iter().next().expect("checked not empty"))
                } else {
                    Inline::CitationGroup(CitationGroup::new(citations))
                };

                blocks.push(Block::InlinesBlock(InlinesBlock::new(vec![inline])));
            }
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
                        } else if name == "results" {
                            // Handle knitr results option: results=hide, results='hide', results="hide"
                            let value = value.trim_matches(|c| c == '\'' || c == '"');
                            if value.eq_ignore_ascii_case("hide")
                                || value.eq_ignore_ascii_case("false")
                            {
                                is_hidden = Some(true);
                            }
                        } else if name == "include" {
                            // Handle knitr include option: include=FALSE hides both code and output
                            if value.eq_ignore_ascii_case("false") {
                                is_echoed = Some(false);
                                is_hidden = Some(true);
                            }
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
        } else if captures.name("printbib").is_some() {
            // Create an empty References section to be populated later
            blocks.push(Block::Section(Section {
                section_type: Some(SectionType::References),
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
