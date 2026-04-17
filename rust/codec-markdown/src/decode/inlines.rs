use std::{collections::HashMap, ops::Range};

use indexmap::IndexMap;
use itertools::Itertools;
use markdown::{mdast, unist::Position};
use winnow::{
    ModalResult, Parser,
    ascii::{multispace0, space0},
    combinator::{alt, delimited, not, opt, peek, preceded, repeat, separated, terminated},
    stream::LocatingSlice as Located,
    token::{take, take_until, take_while},
};

use stencila_codec::{
    stencila_format::Format,
    stencila_schema::{
        AudioObject, BooleanValidator, Boundary, Button, Citation, CitationGroup, CitationMode,
        CodeExpression, CodeInline, Cord, DateTimeValidator, DateValidator, DurationValidator,
        Emphasis, EnumValidator, Icon, ImageObject, Inline, IntegerValidator, Link, MathInline,
        Node, NodeType, Note, NoteType, NumberValidator, Parameter, ParameterOptions, QuoteInline,
        Strikeout, StringValidator, Strong, StyledInline, Subscript, SuggestionInline,
        SuggestionType, Superscript, Text, TimeValidator, TimestampValidator, Underline, Validator,
        VideoObject,
    },
};
use stencila_codec_text_trait::to_text;

use crate::decode::shared::suggestion_metadata_from_attrs;

use super::{
    Context,
    shared::{
        Attrs, attrs, attrs_list, name, node_to_from_str, node_to_option_date,
        node_to_option_datetime, node_to_option_duration, node_to_option_i64,
        node_to_option_number, node_to_option_time, node_to_option_timestamp, node_to_string,
        take_until_unbalanced,
    },
};

pub(super) fn mds_to_inlines(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<Inline> {
    // Collate all the inline nodes, handling merging of inline code with
    // adjacent text for attributes and roles.
    //
    // The markdown crate parses backtick code spans (with `code_text` enabled) but
    // does not handle:
    //   - Stencila/Pandoc-style attributes after code e.g. `code`{python exec}
    //   - MyST roles before code e.g. {eval}`1+1`
    //   - QMD code expressions e.g. `{python} 1+1`
    let mut nodes: Vec<(Inline, Range<usize>)> = Vec::new();

    let mut iter = mds.into_iter().peekable();
    while let Some(md) = iter.next() {
        match md {
            mdast::Node::Text(mdast::Text { value, position }) => {
                // In MyST mode, check for a trailing role like `{eval}` before an InlineCode
                if matches!(context.format, Format::Myst)
                    && let Some(mdast::Node::InlineCode(..)) = iter.peek()
                    && let Some((prefix, role_name)) = extract_trailing_role(&value)
                {
                    // Push any text before the role
                    if !prefix.is_empty() {
                        let span = position
                            .as_ref()
                            .map(|p| p.start.offset..(p.start.offset + prefix.len()))
                            .unwrap_or_default();
                        nodes.push((Inline::Text(Text::from(prefix)), span));
                    }

                    // Consume the following InlineCode
                    let code_node = iter.next().expect("peeked node should exist");
                    let (code_value, code_pos) = match code_node {
                        mdast::Node::InlineCode(mdast::InlineCode { value, position }) => {
                            (value, position)
                        }
                        _ => unreachable!(),
                    };

                    let span = code_pos
                        .map(|p| p.start.offset..p.end.offset)
                        .unwrap_or_default();
                    let inline = apply_myst_role(&role_name, &code_value);
                    nodes.push((inline, span));
                    continue;
                }

                // Parse the text string for extensions not handled by the `markdown` crate e.g.
                // subscripts, superscripts etc and sentinel text like EDIT_END
                let mut parsed = inlines(&value, &context.format)
                    .into_iter()
                    .map(|(inline, span)| {
                        let span = position
                            .as_ref()
                            .map(|p| (p.start.offset + span.start)..(p.start.offset + span.end))
                            .unwrap_or_default();
                        (inline, span)
                    })
                    .collect();
                nodes.append(&mut parsed);
            }

            mdast::Node::InlineCode(mdast::InlineCode { value, position }) => {
                // In QMD mode, check for code expression patterns like `{python} 1+1` or `r 1+1`
                if matches!(context.format, Format::Qmd)
                    && let Some(inline) = try_parse_qmd_code_expression(&value)
                {
                    let span = position
                        .map(|p| p.start.offset..p.end.offset)
                        .unwrap_or_default();
                    nodes.push((inline, span));
                    continue;
                }

                // Check if next node is Text starting with `{...}` for attributes
                if let Some(mdast::Node::Text(mdast::Text {
                    value: text_val, ..
                })) = iter.peek()
                    && let Some(attrs_str) = extract_leading_attrs(text_val)
                {
                    let remaining_text = text_val[attrs_str.len()..].to_string();
                    let text_node = iter.next().expect("peeked node should exist");
                    let text_pos = match &text_node {
                        mdast::Node::Text(t) => t.position.clone(),
                        _ => unreachable!(),
                    };

                    let span = position
                        .as_ref()
                        .map(|p| p.start.offset..p.end.offset)
                        .or_else(|| text_pos.as_ref().map(|p| p.start.offset..p.end.offset))
                        .unwrap_or_default();

                    let inline = apply_code_attrs(&value, &attrs_str);
                    nodes.push((inline, span));

                    // Push any remaining text after the attrs
                    if !remaining_text.is_empty() {
                        let rem_span = text_pos
                            .as_ref()
                            .map(|p| (p.start.offset + attrs_str.len())..p.end.offset)
                            .unwrap_or_default();
                        nodes.push((Inline::Text(Text::from(remaining_text)), rem_span));
                    }
                    continue;
                }

                // Plain inline code
                let span = position
                    .map(|p| p.start.offset..p.end.offset)
                    .unwrap_or_default();
                nodes.push((Inline::CodeInline(CodeInline::new(value.into())), span));
            }

            other => {
                if let Some((inline, position)) = md_to_inline(other, context) {
                    let span = position
                        .map(|p| p.start.offset..p.end.offset)
                        .unwrap_or_default();
                    nodes.push((inline, span));
                }
            }
        }
    }

    // Iterate over the inlines and their spans, adding mapping entries and coalescing where needed
    let mut inlines = Vec::with_capacity(nodes.len());
    for (inline, span) in nodes {
        if let Inline::Text(text) = &inline {
            // Note: currently, mainly for performance reasons, we do not add mapping entries
            // for `Inline::Text` nodes.
            if let Some(Inline::Text(previous_text)) = inlines.last_mut() {
                // The previous inline was text so merge the two
                previous_text.value.push_str(&text.value);
            } else {
                // Just a plain text node so just map and push
                inlines.push(inline);
            }
        } else {
            // Some other inline that does not need a boundary
            context.map_span(span, inline.node_type(), inline.node_id());
            inlines.push(inline)
        }
    }

    inlines
}

/// Extract a leading `{...}` attribute string from text.
///
/// Returns `Some(attrs_str)` including the braces if the text starts with `{...}`,
/// or `None` otherwise.
fn extract_leading_attrs(text: &str) -> Option<String> {
    if !text.starts_with('{') {
        return None;
    }
    // Find the matching closing brace
    let mut depth = 0;
    for (i, ch) in text.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[..=i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract a trailing `{...}` role name from text (for MyST roles).
///
/// Returns `Some((prefix, role_name))` where `prefix` is the text before the role,
/// and `role_name` is the content inside the braces (without braces).
fn extract_trailing_role(text: &str) -> Option<(&str, String)> {
    if !text.ends_with('}') {
        return None;
    }
    // Find the matching opening brace scanning backwards
    if let Some(open) = text.rfind('{') {
        let role_name = text[open + 1..text.len() - 1].to_string();
        if !role_name.is_empty() && !role_name.contains('{') && !role_name.contains('}') {
            let prefix = &text[..open];
            return Some((prefix, role_name));
        }
    }
    None
}

/// Apply attributes from a `{...}` string to an inline code value.
///
/// Reuses the existing `attrs` winnow parser to ensure consistent handling of
/// commas, key=value pairs, and valid identifier characters.
///
/// Produces a `CodeInline`, `CodeExpression`, or `MathInline` depending on the attributes.
fn apply_code_attrs(code: &str, attrs_str: &str) -> Inline {
    // Parse the attrs string using the existing winnow `attrs` parser
    let options = attrs.parse(Located::new(attrs_str)).unwrap_or_default();

    if options.is_empty() {
        return Inline::CodeInline(CodeInline {
            code: code.into(),
            ..Default::default()
        });
    }

    let mut lang = None;
    let mut exec = false;
    let mut execution_mode = None;

    for (name, value) in options {
        if name == "exec" {
            exec = true
        } else if matches!(name, "always" | "auto" | "need" | "lock") && value.is_none() {
            execution_mode = name.parse().ok()
        } else if lang.is_none() && value.is_none() {
            lang = Some(name.to_string());
        }
    }

    if exec {
        Inline::CodeExpression(CodeExpression {
            code: code.into(),
            programming_language: lang,
            execution_mode,
            ..Default::default()
        })
    } else if matches!(
        lang.as_deref(),
        Some("asciimath") | Some("math") | Some("mathml") | Some("latex") | Some("tex")
    ) {
        Inline::MathInline(MathInline {
            code: code.into(),
            math_language: lang,
            ..Default::default()
        })
    } else {
        Inline::CodeInline(CodeInline {
            code: code.into(),
            programming_language: lang,
            ..Default::default()
        })
    }
}

/// Try to parse an inline code value as a QMD code expression.
///
/// Matches patterns like `{python} 1 + 1` or `r 1 + 1`.
fn try_parse_qmd_code_expression(value: &str) -> Option<Inline> {
    // Pattern: {lang} code
    if value.starts_with('{')
        && let Some(close) = value.find('}')
    {
        let lang = &value[1..close];
        // Language must be word characters only
        if !lang.is_empty() && lang.chars().all(|c| c.is_alphanumeric() || c == '_') {
            let code = value[close + 1..].trim();
            if !code.is_empty() {
                return Some(Inline::CodeExpression(CodeExpression {
                    programming_language: Some(lang.to_string()),
                    code: code.into(),
                    ..Default::default()
                }));
            }
        }
    }

    // Pattern: r code (R shorthand)
    if let Some(code) = value.strip_prefix("r ") {
        let code = code.trim();
        if !code.is_empty() {
            return Some(Inline::CodeExpression(CodeExpression {
                programming_language: Some("r".to_string()),
                code: code.into(),
                ..Default::default()
            }));
        }
    }

    None
}

/// Apply a MyST role to an inline code value.
fn apply_myst_role(role_name: &str, code_value: &str) -> Inline {
    let mut name_and_opts = role_name.trim().split(' ');
    let name = name_and_opts.next().unwrap_or_default();

    if name == "eval" {
        const LANGS: &[&str] = &["javascript", "js", "python", "py", "r", "sql"];
        let mut programming_language = None;
        for option in name_and_opts {
            if LANGS.contains(&option.to_lowercase().as_str()) {
                programming_language = Some(option.to_string());
                break;
            }
            let trimmed = option.trim_start_matches('.');
            if LANGS.contains(&trimmed.to_lowercase().as_str()) {
                programming_language = Some(trimmed.to_string());
                break;
            }
        }

        Inline::CodeExpression(CodeExpression {
            code: code_value.into(),
            programming_language,
            ..Default::default()
        })
    } else {
        // Unrecognized role: reconstitute as plain text
        Inline::Text(Text::from(
            &["{", role_name, "}`", code_value, "`"].concat(),
        ))
    }
}

/// Transform MDAST inline nodes to Stencila inlines nodes
fn md_to_inline(md: mdast::Node, context: &mut Context) -> Option<(Inline, Option<Position>)> {
    Some(match md {
        mdast::Node::Delete(mdast::Delete { children, position }) => (
            Inline::Strikeout(Strikeout::new(mds_to_inlines(children, context))),
            position,
        ),

        mdast::Node::Emphasis(mdast::Emphasis { children, position }) => (
            Inline::Emphasis(Emphasis::new(mds_to_inlines(children, context))),
            position,
        ),

        mdast::Node::FootnoteReference(mdast::FootnoteReference {
            identifier,
            label,
            position,
        }) => {
            if label.as_deref() != Some(&identifier) {
                context.lost("FootnoteReference.label")
            }
            let node = Note {
                id: Some(identifier),
                note_type: NoteType::Footnote,
                content: vec![],
                ..Default::default()
            };
            (Inline::Note(node), position)
        }

        // Note: InlineCode is handled directly in mds_to_inlines for attribute/role merging
        mdast::Node::InlineCode(mdast::InlineCode { value, position }) => {
            (Inline::CodeInline(CodeInline::new(value.into())), position)
        }

        mdast::Node::InlineMath(mdast::InlineMath { value, position }) => (
            Inline::MathInline(MathInline {
                code: value.into(),
                math_language: Some("tex".into()),
                ..Default::default()
            }),
            position,
        ),

        mdast::Node::Image(mdast::Image {
            url: content_url,
            alt,
            title,
            position,
        }) => {
            let title = title.map(|title| vec![Inline::Text(Text::from(title))]);
            let caption = (!alt.is_empty()).then_some(vec![Inline::Text(Text::from(alt))]);

            let format = Format::from_url(&content_url);
            let inline = if format.is_audio() {
                Inline::AudioObject(AudioObject {
                    content_url,
                    caption,
                    title,
                    ..Default::default()
                })
            } else if format.is_video() {
                Inline::VideoObject(VideoObject {
                    content_url,
                    caption,
                    title,
                    ..Default::default()
                })
            } else {
                Inline::ImageObject(ImageObject {
                    content_url,
                    caption,
                    title,
                    ..Default::default()
                })
            };

            (inline, position)
        }

        mdast::Node::Link(mdast::Link {
            children,
            url,
            title,
            position,
        }) => (
            Inline::Link(Link {
                target: url,
                title,
                content: mds_to_inlines(children, context),
                ..Default::default()
            }),
            position,
        ),

        mdast::Node::Strong(mdast::Strong { children, position }) => (
            Inline::Strong(Strong::new(mds_to_inlines(children, context))),
            position,
        ),

        mdast::Node::Text(mdast::Text { value, position }) => {
            // This should not be reach because plain text nodes are handled elsewhere
            // but it case it is, return it so not lost
            (Inline::Text(Text::from(value)), position)
        }

        _ => {
            // TODO: Any unexpected blocks should be decomposed to their inline children
            context.lost("Inline");
            return None;
        }
    })
}

/// Parse a text string into a vector of `Inline` nodes with spans
pub(super) fn inlines(input: &str, format: &Format) -> Vec<(Inline, Range<usize>)> {
    let common = alt((
        code_attrs,
        double_braces,
        suggestion_inline,
        comment_boundary,
        citation_group,
        citation_parenthetical,
        citation_narrative,
        parameter,
        button,
        icon,
        styled_inline,
        quote,
        strikeout,
        subscript,
        superscript,
        underline,
        html,
        string,
        character,
    ));

    let text = |_| vec![(Inline::Text(Text::from(input)), 0..input.len())];
    let located = Located::new(input);
    match format {
        Format::Myst => repeat(0.., alt((myst_role, common)).with_span()).parse(located),
        Format::Qmd => repeat(0.., alt((code_expression_qmd, common)).with_span()).parse(located),
        _ => repeat(0.., common.with_span()).parse(located),
    }
    .unwrap_or_else(text)
}

/// Parse a text string into a vector of `Inline` nodes (without spans)
///
/// Used for nested inline content (e.g., inside strong or emphasis) where format specific
/// syntax e.g. QMD-style code expressions are not expected.
fn inlines_only(input: &str) -> Vec<Inline> {
    // Use default format (non-QMD) for nested content
    inlines(input, &Format::default())
        .into_iter()
        .map(|(inlines, ..)| inlines)
        .collect()
}

// Parse a MyST "role" into an inline
fn myst_role(input: &mut Located<&str>) -> ModalResult<Inline> {
    (
        delimited('{', take_until(0.., '}'), '}'),
        delimited('`', take_until(0.., '`'), '`'),
    )
        .map(|(name_and_options, value): (&str, &str)| {
            let mut name_and_opts = name_and_options.trim().split(" ");
            let name = name_and_opts.next().unwrap_or_default();
            if name == "eval" {
                const LANGS: &[&str] = &["javascript", "js", "python", "py", "r", "sql"];
                let mut programming_language = None;
                for option in name_and_opts {
                    // Allow for language options e.g. {eval python}`1 + 2`
                    if LANGS.contains(&option.to_lowercase().as_str()) {
                        programming_language = Some(option.to_string());
                        break;
                    }
                    // Allow for "class style" language option e.g. {eval .python}`1 + 2`
                    if LANGS.contains(&[".", option].concat().as_str()) {
                        programming_language = Some(option.trim_start_matches(".").to_string());
                        break;
                    }
                }

                Inline::CodeExpression(CodeExpression {
                    code: value.into(),
                    programming_language,
                    ..Default::default()
                })
            } else {
                // If the name is not recognized then reconstitute the input as plain text
                Inline::Text(Text::from(
                    &["{", name_and_options, "}`", value, "`"].concat(),
                ))
            }
        })
        .parse_next(input)
}

/// Parse a QMD inline expression (aka inline code) e.g. `{python} 1 + 1` or `r 1 + 1`
/// Only matches if language is word characters only (no whitespace/symbols)
/// and there is non-empty code content after the language.
fn code_expression_qmd(input: &mut Located<&str>) -> ModalResult<Inline> {
    (
        alt((
            delimited(
                "`{",
                take_while(1.., |c: char| c.is_alphanumeric() || c == '_'),
                '}',
            ),
            "`r ".value("r"),
        )),
        delimited(multispace0, take_while(1.., |c| c != '`'), '`'),
    )
        .map(|(lang, value): (&str, &str)| {
            Inline::CodeExpression(CodeExpression {
                programming_language: (!lang.is_empty()).then_some(lang.to_string()),
                code: value.into(),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse inline code with optional attributes in curly braces e.g. `\`code\`{attr1 attr2}`
/// into a `CodeFragment`, `CodeExpression` or `MathFragment` node.
///
/// The `attrs` are optional because plain `CodeFragment`s also end up being
/// passed to this function
fn code_attrs(input: &mut Located<&str>) -> ModalResult<Inline> {
    preceded(
        not(peek(('=', space0))), // Avoid matching call arguments using backticks
        (delimited('`', take_until(0.., '`'), '`'), opt(attrs)),
    )
    .map(|(code, options)| {
        let Some(options) = options else {
            return Inline::CodeInline(CodeInline {
                code: code.into(),
                ..Default::default()
            });
        };

        let mut lang = None;
        let mut exec = false;
        let mut execution_mode = None;

        for (name, value) in options {
            if name == "exec" {
                exec = true
            } else if matches!(name, "always" | "auto" | "need" | "lock") && value.is_none() {
                execution_mode = name.parse().ok()
            } else if lang.is_none() && value.is_none() {
                lang = Some(name.to_string());
            }
        }

        if exec {
            Inline::CodeExpression(CodeExpression {
                code: code.into(),
                programming_language: lang,
                execution_mode,
                ..Default::default()
            })
        } else if matches!(
            lang.as_deref(),
            Some("asciimath") | Some("math") | Some("mathml") | Some("latex") | Some("tex")
        ) {
            Inline::MathInline(MathInline {
                code: code.into(),
                math_language: lang,
                ..Default::default()
            })
        } else {
            Inline::CodeInline(CodeInline {
                code: code.into(),
                programming_language: lang,
                ..Default::default()
            })
        }
    })
    .parse_next(input)
}

/// Parse double brace surrounded text into a `CodeExpression`.
///
/// This supports the Jinja and Jupyter "Python Markdown" extension syntax for
/// interpolated variables / expressions: `{{ x }}`
///
/// Does not support the single curly brace syntax (as in Python, Rust and JSX) i.e. `{ x }`
/// given that is less specific and could conflict with other user content.
///
/// Does not support JavaScript style "dollared-brace" syntax i.e. `${x}` since some
/// at least some Markdown parsers seem to parse that as TeX math (even though there
/// is no closing brace).
///
/// The language for double brace expressions is always DocsQL (an extension of Jinja).
fn double_braces(input: &mut Located<&str>) -> ModalResult<Inline> {
    (delimited("{{", take_until(0.., "}}"), "}}"))
        .map(|code: &str| {
            Inline::CodeExpression(CodeExpression {
                code: code.into(),
                programming_language: Some("docsql".to_string()),
                ..Default::default()
            })
        })
        .parse_next(input)
}

fn citation_target(input: &mut Located<&str>) -> ModalResult<String> {
    preceded(
        '@',
        take_while(1.., |chr: char| {
            // Permissive on target id to allow for the large variety of characters
            // that are permitted in a DOI
            !(chr.is_whitespace() || ['@', ';', '[', ']'].contains(&chr))
        }),
    )
    .map(|target: &str| target.to_string())
    .parse_next(input)
}

/// Parse a string into a narrative `Citation` node
fn citation_narrative(input: &mut Located<&str>) -> ModalResult<Inline> {
    citation_target
        .map(|target| {
            Inline::Citation(Citation {
                target,
                citation_mode: Some(CitationMode::Narrative),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a string into a parenthetical `Citation` node
///
/// This attempts to follow Pandoc's citation handling as closely as possible
/// (see <https://pandoc.org/MANUAL.html#citations>).
///
/// The following properties of a `Citation` are parsed:
///   - [x] target
///   - [ ] citation_mode
///   - [ ] page_start
///   - [ ] page_end
///   - [ ] pagination
///   - [ ] citation_prefix
///   - [ ] citation_suffix
///   - [ ] citation_intent
fn citation_parenthetical(input: &mut Located<&str>) -> ModalResult<Inline> {
    delimited(('[', multispace0), citation_target, (multispace0, ']'))
        .map(|target| {
            Inline::Citation(Citation {
                target,
                citation_mode: Some(CitationMode::Parenthetical),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a string into a `CitationGroup` node or parenthetical `Citation` node.
///
/// If there is only one citation within square brackets then a parenthetical `Citation` node is
/// returned. Otherwise, the `Citation` nodes are grouped into into a `CitationGroup`.
fn citation_group(input: &mut Located<&str>) -> ModalResult<Inline> {
    let cite = citation_target.map(|target| {
        Inline::Citation(Citation {
            target,
            ..Default::default()
        })
    });

    delimited(
        '[',
        separated(2.., cite, (multispace0, ';', multispace0)),
        ']',
    )
    .map(|items: Vec<Inline>| {
        Inline::CitationGroup(CitationGroup {
            items: items
                .iter()
                .filter_map(|item| match item {
                    Inline::Citation(cite) => Some(cite),
                    _ => None,
                })
                .cloned()
                .collect(),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse a `Parameter`.
fn parameter(input: &mut Located<&str>) -> ModalResult<Inline> {
    (delimited("&[", name, ']'), opt(attrs))
        .map(|(name, attrs)| {
            let mut options: HashMap<&str, _> = attrs.unwrap_or_default().into_iter().collect();

            // Properties for parameters, regardless of validator type
            let label = options.remove("label").flatten().map(node_to_string);
            let default = options
                .remove("default")
                .or_else(|| options.remove("def"))
                .flatten();

            // The type of validator: either using the explicit `type=` or the attribute
            // that does not have a value e.g `bool`, `num`
            let validator_type = options
                .remove("type")
                .flatten()
                .map(node_to_string)
                .or_else(|| {
                    options
                        .iter()
                        .find_map(|(key, value)| value.is_none().then_some(key.to_string()))
                });

            // Properties for many types of validators
            let minimum = options.remove("min").flatten();
            let maximum = options.remove("max").flatten();

            // Properties for `EnumValidator`
            let values = options.remove("vals").flatten();

            // Properties for `IntegerValidator` and `NumberValidator`
            let exclusive_minimum = options.remove("emin").flatten();
            let exclusive_maximum = options.remove("emax").flatten();
            let multiple_of = options
                .remove("mult")
                .or_else(|| options.remove("step"))
                .flatten();

            // Properties for `StringValidator`
            let min_length = options
                .remove("minlen")
                .or_else(|| options.remove("minlength"))
                .flatten();
            let max_length = options
                .remove("maxlen")
                .or_else(|| options.remove("maxlength"))
                .flatten();
            let pattern = options
                .remove("pattern")
                .or_else(|| options.remove("regex"))
                .flatten();

            // If the validator type is specified with a string, map that to the actual type.
            // If it is not specified, attempt to infer it from other options.
            let validator_type = if let Some(validator_type) = validator_type {
                match validator_type.to_lowercase().as_str() {
                    "bool" | "boolean" => Some(NodeType::BooleanValidator),
                    "enum" => Some(NodeType::EnumValidator),
                    "int" | "integer" => Some(NodeType::IntegerValidator),
                    "num" | "number" => Some(NodeType::NumberValidator),
                    "str" | "string" => Some(NodeType::StringValidator),
                    "date" => Some(NodeType::DateValidator),
                    "time" => Some(NodeType::TimeValidator),
                    "datetime" => Some(NodeType::DateTimeValidator),
                    "timestamp" => Some(NodeType::TimestampValidator),
                    "duration" => Some(NodeType::DurationValidator),
                    _ => {
                        tracing::warn!("Unknown parameter type `{validator_type}`");
                        None
                    }
                }
            } else if min_length.is_some() || max_length.is_some() || pattern.is_some() {
                Some(NodeType::StringValidator)
            } else if let Some(node) = default
                .as_ref()
                .or(minimum.as_ref())
                .or(maximum.as_ref())
                .or(exclusive_minimum.as_ref())
                .or(exclusive_maximum.as_ref())
                .or(multiple_of.as_ref())
            {
                match node {
                    Node::Boolean(..) => Some(NodeType::BooleanValidator),
                    Node::Integer(..) => Some(NodeType::IntegerValidator),
                    Node::Number(..) => Some(NodeType::NumberValidator),
                    Node::String(..) => Some(NodeType::StringValidator),
                    Node::Date(..) => Some(NodeType::DateValidator),
                    Node::Time(..) => Some(NodeType::TimeValidator),
                    Node::DateTime(..) => Some(NodeType::DateTimeValidator),
                    Node::Timestamp(..) => Some(NodeType::TimestampValidator),
                    Node::Duration(..) => Some(NodeType::DurationValidator),
                    _ => {
                        tracing::warn!(
                            "Unable to infer parameter type from default, min or max value"
                        );
                        None
                    }
                }
            } else {
                None
            };

            // Map the validator type into a validator
            let validator = validator_type.and_then(|vt| match vt {
                NodeType::BooleanValidator => {
                    Some(Validator::BooleanValidator(BooleanValidator::default()))
                }
                NodeType::EnumValidator => {
                    let values = values
                        .map(|node| {
                            match node {
                                // Usually the supplied node is an array, which we need to convert
                                // to a vector of `Node`s
                                Node::Array(array) => array
                                    .iter()
                                    .map(|primitive| primitive.clone().into())
                                    .collect(),
                                _ => vec![node],
                            }
                        })
                        .unwrap_or_default();
                    Some(Validator::EnumValidator(EnumValidator {
                        values,
                        ..Default::default()
                    }))
                }
                NodeType::IntegerValidator => Some(Validator::IntegerValidator(IntegerValidator {
                    minimum: minimum.and_then(node_to_option_number),
                    exclusive_minimum: exclusive_minimum.and_then(node_to_option_number),
                    maximum: maximum.and_then(node_to_option_number),
                    exclusive_maximum: exclusive_maximum.and_then(node_to_option_number),
                    multiple_of: multiple_of.and_then(node_to_option_number),
                    ..Default::default()
                })),
                NodeType::NumberValidator => Some(Validator::NumberValidator(NumberValidator {
                    minimum: minimum.and_then(node_to_option_number),
                    exclusive_minimum: exclusive_minimum.and_then(node_to_option_number),
                    maximum: maximum.and_then(node_to_option_number),
                    exclusive_maximum: exclusive_maximum.and_then(node_to_option_number),
                    multiple_of: multiple_of.and_then(node_to_option_number),
                    ..Default::default()
                })),
                NodeType::StringValidator => Some(Validator::StringValidator(StringValidator {
                    min_length: min_length.and_then(node_to_option_i64),
                    max_length: max_length.and_then(node_to_option_i64),
                    pattern: pattern.map(node_to_string),
                    ..Default::default()
                })),
                NodeType::DateValidator => Some(Validator::DateValidator(DateValidator {
                    minimum: minimum.and_then(node_to_option_date),
                    maximum: maximum.and_then(node_to_option_date),
                    ..Default::default()
                })),
                NodeType::TimeValidator => Some(Validator::TimeValidator(TimeValidator {
                    minimum: minimum.and_then(node_to_option_time),
                    maximum: maximum.and_then(node_to_option_time),
                    ..Default::default()
                })),
                NodeType::DateTimeValidator => {
                    Some(Validator::DateTimeValidator(DateTimeValidator {
                        minimum: minimum.and_then(node_to_option_datetime),
                        maximum: maximum.and_then(node_to_option_datetime),
                        ..Default::default()
                    }))
                }
                NodeType::TimestampValidator => {
                    Some(Validator::TimestampValidator(TimestampValidator {
                        minimum: minimum.and_then(node_to_option_timestamp),
                        maximum: maximum.and_then(node_to_option_timestamp),
                        ..Default::default()
                    }))
                }
                NodeType::DurationValidator => {
                    Some(Validator::DurationValidator(DurationValidator {
                        minimum: minimum.and_then(node_to_option_duration),
                        maximum: maximum.and_then(node_to_option_duration),
                        ..Default::default()
                    }))
                }
                _ => None,
            });

            Inline::Parameter(Parameter {
                name: name.into(),
                options: Box::new(ParameterOptions {
                    label,
                    validator,
                    default: default.map(Box::new),
                    ..Default::default()
                }),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a `Button`
fn button(input: &mut Located<&str>) -> ModalResult<Inline> {
    (
        delimited("#[", take_until(0.., ']'), ']'),
        opt(delimited('`', take_until(0.., "`"), '`')),
        opt(attrs),
    )
        .map(|(name, condition, options)| {
            let mut options: IndexMap<&str, Option<Node>> =
                options.unwrap_or_default().into_iter().collect();

            Inline::Button(Button {
                name: name.to_string(),
                code: condition.map_or_else(Cord::default, Cord::from),
                programming_language: options.first().map(|(lang, ..)| lang.to_string()),
                label: options.swap_remove("label").flatten().map(node_to_string),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse `%[name]` or `%[name]{attrs}` into an `Icon` node
fn icon(input: &mut Located<&str>) -> ModalResult<Inline> {
    let icon_name: &str = delimited("%[", take_until(1.., ']'), ']').parse_next(input)?;

    let name = icon_name.trim();
    if name.is_empty() {
        return winnow::combinator::fail.parse_next(input);
    }

    let mut style = None;
    let mut label = None;
    let mut decorative = None;

    if let Ok(options) = attrs.parse_next(input) {
        for (key, value) in options {
            match key {
                "style" => {
                    style = value.map(node_to_string);
                }
                "label" => {
                    label = value.map(node_to_string);
                }
                "decorative" => {
                    decorative = value.and_then(node_to_from_str::<bool>);
                }
                _ => {
                    if value.is_none() {
                        match &mut style {
                            Some(style) => {
                                style.push(' ');
                                style.push_str(key)
                            }
                            None => style = Some(key.into()),
                        }
                    }
                }
            }
        }
    }

    Ok(Inline::Icon(Icon {
        name: name.to_string(),
        style,
        label,
        decorative,
        ..Default::default()
    }))
}

/// Parse a [`StyledInline`].
fn styled_inline(input: &mut Located<&str>) -> ModalResult<Inline> {
    (
        delimited('[', take_until_unbalanced('[', ']'), ']'),
        delimited('{', take_until_unbalanced('{', '}'), '}'),
    )
        .map(|(content, code): (&str, &str)| {
            Inline::StyledInline(StyledInline {
                content: inlines_only(content),
                code: code.into(),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a string into a `SuggestionInline` node using Critic Markup syntax
///
/// Insertions: `{++inserted text++}`
/// Deletions: `{--deleted text--}`
/// Replacements: `{~~old text~>new text~~}`
fn suggestion_inline(input: &mut Located<&str>) -> ModalResult<Inline> {
    alt((
        (
            delimited("{~~", take_until(0.., "~>"), "~>"),
            take_until(0.., "~~}"),
            "~~}",
            opt(attrs),
        )
            .map(
                |(original, content, _, attrs): (&str, &str, &str, Option<Attrs>)| {
                    let (authors, date_published) = suggestion_metadata_from_attrs(attrs);

                    Inline::SuggestionInline(SuggestionInline {
                        suggestion_type: Some(SuggestionType::Replace),
                        content: inlines_only(content),
                        original: Some(inlines_only(original)),
                        authors,
                        date_published,
                        ..Default::default()
                    })
                },
            ),
        (delimited("{++", take_until(0.., "++}"), "++}"), opt(attrs)).map(
            |(content, attrs): (&str, Option<Attrs>)| {
                let (authors, date_published) = suggestion_metadata_from_attrs(attrs);

                Inline::SuggestionInline(SuggestionInline {
                    suggestion_type: Some(SuggestionType::Insert),
                    content: inlines_only(content),
                    authors,
                    date_published,
                    ..Default::default()
                })
            },
        ),
        (delimited("{--", take_until(0.., "--}"), "--}"), opt(attrs)).map(
            |(content, attrs): (&str, Option<Attrs>)| {
                let (authors, date_published) = suggestion_metadata_from_attrs(attrs);

                Inline::SuggestionInline(SuggestionInline {
                    suggestion_type: Some(SuggestionType::Delete),
                    content: inlines_only(content),
                    authors,
                    date_published,
                    ..Default::default()
                })
            },
        ),
    ))
    .parse_next(input)
}

/// Parse a comment boundary marker `{>>id}` (start) or `{<<id}` (end)
fn comment_boundary(input: &mut Located<&str>) -> ModalResult<Inline> {
    alt((
        delimited(
            "{>>",
            take_while(1.., |c: char| {
                c.is_alphanumeric() || c == '.' || c == '-' || c == '_'
            }),
            "}",
        )
        .map(|id: &str| {
            Inline::Boundary(Boundary {
                id: Some(format!("comment-{id}-start")),
                ..Default::default()
            })
        }),
        delimited(
            "{<<",
            take_while(1.., |c: char| {
                c.is_alphanumeric() || c == '.' || c == '-' || c == '_'
            }),
            "}",
        )
        .map(|id: &str| {
            Inline::Boundary(Boundary {
                id: Some(format!("comment-{id}-end")),
                ..Default::default()
            })
        }),
    ))
    .parse_next(input)
}

/// Parse a string into a `Strikeout` node
fn strikeout(input: &mut Located<&str>) -> ModalResult<Inline> {
    alt((
        delimited("~~", take_until(0.., "~~"), "~~"),
        delimited("<s>", take_until(0.., "</s>"), "</s>"),
        delimited("<del>", take_until(0.., "</del>"), "</del>"),
    ))
    .map(|content: &str| Inline::Strikeout(Strikeout::new(inlines_only(content))))
    .parse_next(input)
}

/// Parse a string into a `Subscript` node
fn subscript(input: &mut Located<&str>) -> ModalResult<Inline> {
    alt((
        delimited(
            // Only match single tilde, because doubles are for `Strikeout`, Do not allow whitespace.
            ('~', peek(not('~'))),
            take_while(1.., |c: char| c != '~' && !c.is_whitespace()),
            '~',
        ),
        delimited("<sub>", take_until(0.., "</sub>"), "</sub>"),
    ))
    .map(|content: &str| Inline::Subscript(Subscript::new(inlines_only(content))))
    .parse_next(input)
}

/// Parse a string into a `Superscript` node
fn superscript(input: &mut Located<&str>) -> ModalResult<Inline> {
    alt((
        // Do not allow whitespace in superscript
        delimited(
            '^',
            take_while(1.., |c: char| c != '^' && !c.is_whitespace()),
            '^',
        ),
        delimited("<sup>", take_until(0.., "</sup>"), "</sup>"),
    ))
    .map(|content: &str| Inline::Superscript(Superscript::new(inlines_only(content))))
    .parse_next(input)
}

/// Nest HTML tag only parsers under a peek for performance (avoids trying each one in the input does not start with <)
fn html(input: &mut Located<&str>) -> ModalResult<Inline> {
    preceded(
        peek("<"),
        alt((
            quote, underline, emphasis, strong, code, image, link, html_tag,
        )),
    )
    .parse_next(input)
}

/// Parse <q> tags into a `QuoteInline` node
fn quote(input: &mut Located<&str>) -> ModalResult<Inline> {
    delimited("<q>", take_until(0.., "</q>"), "</q>")
        .map(|content: &str| Inline::QuoteInline(QuoteInline::new(inlines_only(content))))
        .parse_next(input)
}

/// Parse <u> tags into a `Underline` node
fn underline(input: &mut Located<&str>) -> ModalResult<Inline> {
    delimited("<u>", take_until(0.., "</u>"), "</u>")
        .map(|content: &str| Inline::Underline(Underline::new(inlines_only(content))))
        .parse_next(input)
}

/// Parse <em> tags into a `Emphasis` node
fn emphasis(input: &mut Located<&str>) -> ModalResult<Inline> {
    delimited("<em>", take_until(0.., "</em>"), "</em>")
        .map(|content: &str| Inline::Emphasis(Emphasis::new(inlines_only(content))))
        .parse_next(input)
}

/// Parse <strong> tags into a `Strong` node
fn strong(input: &mut Located<&str>) -> ModalResult<Inline> {
    delimited("<strong>", take_until(0.., "</strong>"), "</strong>")
        .map(|content: &str| Inline::Strong(Strong::new(inlines_only(content))))
        .parse_next(input)
}

/// Parse <code> tags into a `CodeInline` node
fn code(input: &mut Located<&str>) -> ModalResult<Inline> {
    delimited("<code>", take_until(0.., "</code>"), "</code>")
        .map(|code: &str| Inline::CodeInline(CodeInline::new(code.into())))
        .parse_next(input)
}

/// Parse <img> tags into a `Image` node
fn image(input: &mut Located<&str>) -> ModalResult<Inline> {
    delimited("<img ", attrs_list, ">")
        .map(|attrs| {
            let mut content_url = String::new();
            let mut title = None;
            for (key, value) in attrs {
                if key == "src" {
                    content_url = to_text(&value);
                } else if key == "title" {
                    title = Some(inlines_only(&to_text(&value)));
                }
            }

            Inline::ImageObject(ImageObject {
                content_url,
                title,
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse <a> tags into a `Link` node
fn link(input: &mut Located<&str>) -> ModalResult<Inline> {
    delimited(
        "<a ",
        (terminated(attrs_list, ">"), take_until(0.., "</a>")),
        "</a>",
    )
    .map(|(attrs, content)| {
        let mut target = String::new();
        let mut title = None;
        for (key, value) in attrs {
            if key == "href" {
                target = to_text(&value);
            } else if key == "title" {
                title = Some(to_text(&value));
            }
        }

        Inline::Link(Link {
            target,
            title,
            content: inlines_only(content),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Ignore other inline HTML start and end tags. The content between them will still be parsed elsewhere
/// so this does not try to balance them
fn html_tag(input: &mut Located<&str>) -> ModalResult<Inline> {
    alt((
        delimited(
            "</",
            take_while(1.., |c: char| c.is_ascii_alphabetic()),
            ">",
        ),
        delimited("<", take_while(1.., |c: char| c != '>'), ">"),
    ))
    .map(|_| Inline::Text(Text::from("")))
    .parse_next(input)
}

/// Accumulate characters into a `Text` node
///
/// Will greedily take as many characters as possible, excluding those that appear at the
/// start of other inline parsers e.g. '$', '['
fn string(input: &mut Located<&str>) -> ModalResult<Inline> {
    const CHARS: &str = "~@#$^&%[]{`<>";
    take_while(1.., |chr: char| !CHARS.contains(chr))
        .map(|val: &str| Inline::Text(Text::new(val.into())))
        .parse_next(input)
}

/// Take a single character into a `Text` node
///
/// Necessary so that the characters not consumed by `string` are not lost.
fn character(input: &mut Located<&str>) -> ModalResult<Inline> {
    take(1usize)
        .map(|val: &str| Inline::Text(Text::new(val.into())))
        .parse_next(input)
}

/// Transform MDAST inline nodes back to a Markdown String
///
/// Attempts, imperfectly, to recreate the string in the document.
/// See call sites for why this is necessary.
pub(super) fn mds_to_string(mds: &[mdast::Node]) -> String {
    mds.iter()
        .map(|md| match md {
            mdast::Node::Delete(mdast::Delete { children, .. }) => {
                ["~~", &mds_to_string(children), "~~"].concat()
            }
            mdast::Node::Emphasis(mdast::Emphasis { children, .. }) => {
                ["_", &mds_to_string(children), "_"].concat()
            }
            mdast::Node::FootnoteReference(mdast::FootnoteReference { identifier, .. }) => {
                ["[^", identifier, "]"].concat()
            }
            mdast::Node::InlineCode(mdast::InlineCode { value, .. }) => ["`", value, "`"].concat(),
            mdast::Node::InlineMath(mdast::InlineMath { value, .. }) => ["$", value, "$"].concat(),
            mdast::Node::Image(mdast::Image { url, alt, .. }) => {
                ["![", alt, "](", url, ")"].concat()
            }
            mdast::Node::Link(mdast::Link { url, .. }) => url.clone(),
            mdast::Node::Strong(mdast::Strong { children, .. }) => {
                ["*", &mds_to_string(children), "*"].concat()
            }
            mdast::Node::Text(mdast::Text { value, .. }) => value.clone(),
            _ => String::new(),
        })
        .join("")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use pretty_assertions::assert_eq;
    use stencila_codec::stencila_schema::ExecutionMode;

    use super::*;

    #[test]
    fn test_cite() {
        assert_eq!(
            citation_narrative(&mut Located::new("@someref")).unwrap(),
            Inline::Citation(Citation {
                target: "someref".into(),
                citation_mode: Some(CitationMode::Narrative),
                ..Default::default()
            })
        );

        assert_eq!(
            citation_narrative(&mut Located::new("@10.0000/a-b_c/e#123")).unwrap(),
            Inline::Citation(Citation {
                target: "10.0000/a-b_c/e#123".into(),
                citation_mode: Some(CitationMode::Narrative),
                ..Default::default()
            })
        );

        assert_eq!(
            citation_parenthetical(&mut Located::new("[@10.0000/2020.10.10(321)#123]")).unwrap(),
            Inline::Citation(Citation {
                target: "10.0000/2020.10.10(321)#123".into(),
                citation_mode: Some(CitationMode::Parenthetical),
                ..Default::default()
            })
        );

        assert_eq!(
            citation_group(&mut Located::new("[@a; @b; @c ; @d]")).unwrap(),
            Inline::CitationGroup(CitationGroup {
                items: vec![
                    Citation {
                        target: "a".into(),
                        ..Default::default()
                    },
                    Citation {
                        target: "b".into(),
                        ..Default::default()
                    },
                    Citation {
                        target: "c".into(),
                        ..Default::default()
                    },
                    Citation {
                        target: "d".into(),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_code_attrs() {
        code_attrs(&mut Located::new("``")).unwrap();
        code_attrs(&mut Located::new("``{}")).unwrap();
        code_attrs(&mut Located::new("`a + b`{}")).unwrap();
        code_attrs(&mut Located::new("`a + b`{python}")).unwrap();
        code_attrs(&mut Located::new("`a + b`{python exec}")).unwrap();
        code_attrs(&mut Located::new("`a + b`{python exec always}")).unwrap();

        assert_eq!(
            code_attrs(&mut Located::new("`a + b`{javascript exec auto}")).unwrap(),
            Inline::CodeExpression(CodeExpression {
                code: "a + b".into(),
                programming_language: Some("javascript".into()),
                execution_mode: Some(ExecutionMode::Auto),
                ..Default::default()
            })
        );

        assert!(code_attrs(&mut Located::new("=`1*1`")).is_err());
        assert!(code_attrs(&mut Located::new("= `2+2`")).is_err());

        // Two or more dollars in code is OK (previous got split out as math)
        let is = inlines("`${a} ${b}`", &Format::default());
        assert_eq!(is.len(), 1);
        assert_eq!(is[0].0.node_type(), NodeType::CodeInline);
    }

    #[test]
    fn test_subscript() {
        let res = inlines("before H~2~0 after", &Format::default());
        assert_eq!(res.len(), 3);
        assert_eq!(to_text(&res[0].0), "before H");
        assert!(matches!(res[1].0, Inline::Subscript(..)));
        assert_eq!(to_text(&res[1].0), "2");
        assert_eq!(to_text(&res[2].0), "0 after");

        let res = inlines("before <sub>subscripted</sub> after", &Format::default());
        assert_eq!(res.len(), 3);
        assert!(matches!(res[1].0, Inline::Subscript(..)));
        assert_eq!(to_text(&res[1].0), "subscripted");
    }

    #[test]
    fn test_superscript() {
        let res = inlines("before CO^2^ after", &Format::default());
        assert_eq!(res.len(), 3);
        assert_eq!(to_text(&res[0].0), "before CO");
        assert!(matches!(res[1].0, Inline::Superscript(..)));
        assert_eq!(to_text(&res[1].0), "2");
        assert_eq!(to_text(&res[2].0), " after");

        let res = inlines("before <sup>superscripted</sup> after", &Format::default());
        assert_eq!(res.len(), 3);
        assert!(matches!(res[1].0, Inline::Superscript(..)));
        assert_eq!(to_text(&res[1].0), "superscripted");
    }

    #[test]
    fn test_underline() {
        underline(&mut Located::new("<u></u>")).unwrap();
        underline(&mut Located::new("<u>underlined</u>")).unwrap();

        let inlines = inlines("before <u>underlined</u> after", &Format::default());
        assert_eq!(inlines.len(), 3);
        assert!(matches!(inlines[1].0, Inline::Underline(..)));
    }

    #[test]
    fn test_code_expression_qmd() {
        // QMD format: `{python} 1 + 1` should create CodeExpression
        let res = inlines("`{python} 1 + 1`", &Format::Qmd);
        assert_eq!(res.len(), 1);
        assert!(matches!(res[0].0, Inline::CodeExpression(..)));
        if let Inline::CodeExpression(expr) = &res[0].0 {
            assert_eq!(expr.programming_language, Some("python".to_string()));
            assert_eq!(expr.code.as_str(), "1 + 1");
        }

        // Non-QMD format: `{python} 1 + 1` should NOT create CodeExpression
        let res = inlines("`{python} 1 + 1`", &Format::default());
        assert_eq!(res.len(), 1);
        // Should be parsed as CodeInline with code_attrs, not CodeExpression
        assert!(matches!(res[0].0, Inline::CodeInline(..)));

        // Invalid: braces with non-word chars should not match
        let res = inlines("`{foo: bar} + 1`", &Format::Qmd);
        assert_eq!(res.len(), 1);
        // Should fall through to code_attrs and be CodeInline
        assert!(matches!(res[0].0, Inline::CodeInline(..)));

        // Invalid: empty content after braces should not match
        let res = inlines("`{foo}`", &Format::Qmd);
        assert_eq!(res.len(), 1);
        // Should fall through to code_attrs and be CodeInline
        assert!(matches!(res[0].0, Inline::CodeInline(..)));

        // R shorthand: `r expr` should work in QMD mode
        let res = inlines("`r 1 + 1`", &Format::Qmd);
        assert_eq!(res.len(), 1);
        assert!(matches!(res[0].0, Inline::CodeExpression(..)));
        if let Inline::CodeExpression(expr) = &res[0].0 {
            assert_eq!(expr.programming_language, Some("r".to_string()));
            assert_eq!(expr.code.as_str(), "1 + 1");
        }
    }

    /// Helper to decode Markdown and extract the inlines from the first paragraph
    fn decode_para_inlines(md: &str, format: Format) -> Vec<Inline> {
        use stencila_codec::DecodeOptions;
        use stencila_codec::stencila_schema::{Article, Block, Node};

        let options = DecodeOptions {
            format: Some(format),
            ..Default::default()
        };
        let (node, _) = super::super::decode(md, Some(options)).unwrap();
        if let Node::Article(Article { content, .. }) = node
            && let Some(Block::Paragraph(para)) = content.first()
        {
            return para.content.clone();
        }
        vec![]
    }

    #[test]
    fn test_dollars_in_backticks() {
        // Backtick code spans should take precedence over dollar-sign math
        let inlines = decode_para_inlines("Some `$code$`", Format::Smd);
        assert_eq!(inlines.len(), 2);
        assert!(matches!(&inlines[0], Inline::Text(t) if t.value.as_str() == "Some "));
        assert!(matches!(&inlines[1], Inline::CodeInline(c) if c.code.as_str() == "$code$"));

        // Plain dollar math still works
        let inlines = decode_para_inlines("Some $math$", Format::Smd);
        assert_eq!(inlines.len(), 2);
        assert!(matches!(&inlines[0], Inline::Text(t) if t.value.as_str() == "Some "));
        assert!(matches!(&inlines[1], Inline::MathInline(m) if m.code.as_str() == "math"));

        // Code with multiple dollars
        let inlines = decode_para_inlines("`${a} ${b}`", Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeInline(c) if c.code.as_str() == "${a} ${b}"));
    }

    #[test]
    fn test_code_with_attrs_via_markdown() {
        // Code with language attribute
        let inlines = decode_para_inlines("`code`{python}", Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeInline(c)
            if c.code.as_str() == "code"
            && c.programming_language.as_deref() == Some("python")));

        // Code with exec attribute (space-separated)
        let inlines = decode_para_inlines("`a + b`{python exec}", Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeExpression(e)
            if e.code.as_str() == "a + b"
            && e.programming_language.as_deref() == Some("python")));

        // Code with exec attribute (comma-separated)
        let inlines = decode_para_inlines("`a + b`{python, exec}", Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeExpression(e)
            if e.code.as_str() == "a + b"
            && e.programming_language.as_deref() == Some("python")));

        // Code with math language → MathInline
        let inlines = decode_para_inlines("`E = mc^2`{tex}", Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::MathInline(m)
            if m.code.as_str() == "E = mc^2"
            && m.math_language.as_deref() == Some("tex")));

        // Code with empty attrs → plain CodeInline
        let inlines = decode_para_inlines("`code`{}", Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeInline(c) if c.code.as_str() == "code"));

        // Code with attrs followed by text
        let inlines = decode_para_inlines("`code`{python} and more", Format::Smd);
        assert_eq!(inlines.len(), 2);
        assert!(matches!(&inlines[0], Inline::CodeInline(c)
            if c.code.as_str() == "code"
            && c.programming_language.as_deref() == Some("python")));
        assert!(matches!(&inlines[1], Inline::Text(t) if t.value.as_str() == " and more"));
    }

    #[test]
    fn test_qmd_code_expression_via_markdown() {
        let inlines = decode_para_inlines("`{python} 1 + 1`", Format::Qmd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeExpression(e)
            if e.code.as_str() == "1 + 1"
            && e.programming_language.as_deref() == Some("python")));

        let inlines = decode_para_inlines("`r 1 + 1`", Format::Qmd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeExpression(e)
            if e.code.as_str() == "1 + 1"
            && e.programming_language.as_deref() == Some("r")));

        // Non-QMD: should be plain code
        let inlines = decode_para_inlines("`{python} 1 + 1`", Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeInline(..)));
    }

    #[test]
    fn test_myst_role_via_markdown() {
        let inlines = decode_para_inlines("{eval}`1 + 1`", Format::Myst);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeExpression(e)
            if e.code.as_str() == "1 + 1"));

        let inlines = decode_para_inlines("{eval python}`1 + 1`", Format::Myst);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::CodeExpression(e)
            if e.code.as_str() == "1 + 1"
            && e.programming_language.as_deref() == Some("python")));

        // Text before MyST role
        let inlines = decode_para_inlines("Value is {eval}`x`", Format::Myst);
        assert_eq!(inlines.len(), 2);
        assert!(matches!(&inlines[0], Inline::Text(t) if t.value.as_str() == "Value is "));
        assert!(matches!(&inlines[1], Inline::CodeExpression(e) if e.code.as_str() == "x"));

        // Unrecognized role → plain text
        let inlines = decode_para_inlines("{unknown}`value`", Format::Myst);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(&inlines[0], Inline::Text(..)));
    }

    // --- Icon parsing: basic %[name] syntax ---

    #[test]
    fn test_icon_basic_parse() {
        use stencila_codec::stencila_schema::Icon;

        let res = inlines("%[mdi:home]", &Format::default());
        assert_eq!(res.len(), 1);
        assert!(matches!(
            &res[0].0,
            Inline::Icon(Icon { name, .. }) if name == "mdi:home"
        ));
    }

    #[test]
    fn test_icon_parse_tabler() {
        use stencila_codec::stencila_schema::Icon;

        let res = inlines("%[tabler:alert-circle]", &Format::default());
        assert_eq!(res.len(), 1);
        assert!(matches!(
            &res[0].0,
            Inline::Icon(Icon { name, .. }) if name == "tabler:alert-circle"
        ));
    }

    #[test]
    fn test_icon_parse_surrounded_by_text() {
        let res = inlines("before %[mdi:home] after", &Format::default());
        assert_eq!(res.len(), 3);
        assert_eq!(to_text(&res[0].0), "before ");
        assert!(matches!(&res[1].0, Inline::Icon(..)));
        assert_eq!(to_text(&res[2].0), " after");
    }

    #[test]
    fn test_icon_empty_name_not_parsed() {
        let res = inlines("%[]", &Format::default());
        // Should NOT produce an Icon — falls through to text
        assert!(
            !res.iter()
                .any(|(inline, _)| matches!(inline, Inline::Icon(..)))
        );
    }

    #[test]
    fn test_icon_whitespace_only_name_not_parsed() {
        let res = inlines("%[  ]", &Format::default());
        // Should NOT produce an Icon — falls through to text
        assert!(
            !res.iter()
                .any(|(inline, _)| matches!(inline, Inline::Icon(..)))
        );
    }

    #[test]
    fn test_icon_leading_trailing_whitespace_trimmed() {
        use stencila_codec::stencila_schema::Icon;

        let res = inlines("%[ mdi:home ]", &Format::default());
        assert_eq!(res.len(), 1);
        assert!(matches!(
            &res[0].0,
            Inline::Icon(Icon { name, .. }) if name == "mdi:home"
        ));
    }

    // --- Icon parsing: extended %[name]{attrs} syntax ---

    #[test]
    fn test_icon_with_label_attr() {
        use stencila_codec::stencila_schema::Icon;

        let res = inlines(r#"%[mdi:home]{label="Home"}"#, &Format::default());
        assert_eq!(res.len(), 1);
        assert!(matches!(
            &res[0].0,
            Inline::Icon(Icon { name, label: Some(label), decorative: None, .. })
                if name == "mdi:home" && label == "Home"
        ));
    }

    #[test]
    fn test_icon_with_decorative_attr() {
        use stencila_codec::stencila_schema::Icon;

        let res = inlines("%[mdi:home]{decorative=false}", &Format::default());
        assert_eq!(res.len(), 1);
        assert!(matches!(
            &res[0].0,
            Inline::Icon(Icon { name, decorative: Some(false), .. })
                if name == "mdi:home"
        ));
    }

    #[test]
    fn test_icon_with_label_and_decorative_attrs() {
        use stencila_codec::stencila_schema::Icon;

        let res = inlines(
            r#"%[mdi:home]{label="Home" decorative=false}"#,
            &Format::default(),
        );
        assert_eq!(res.len(), 1);
        assert!(matches!(
            &res[0].0,
            Inline::Icon(Icon { name, label: Some(label), decorative: Some(false), .. })
                if name == "mdi:home" && label == "Home"
        ));
    }

    // --- Icon parsing: negative / edge-case tests for attrs ---

    #[test]
    fn test_icon_malformed_attrs_falls_through() {
        // Unterminated attrs block — the icon name should still parse but the
        // trailing `{label=...` should be separate text, or the whole thing
        // falls through. Either way, no panic and no malformed Icon.
        let res = inlines(r#"%[mdi:home]{label="Home"#, &Format::default());
        // Should not produce an Icon with a valid label from malformed attrs
        for (inline, _) in &res {
            if let Inline::Icon(icon) = inline {
                // If an Icon was parsed (just the name part), that's fine,
                // but it should NOT have extracted a label from malformed attrs
                assert!(icon.label.is_none());
            }
        }
    }

    // --- Icon parsing via full decode pipeline ---

    #[test]
    fn test_icon_basic_via_decode() {
        use stencila_codec::stencila_schema::Icon;

        let inlines = decode_para_inlines("%[mdi:home]", Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(
            &inlines[0],
            Inline::Icon(Icon { name, .. }) if name == "mdi:home"
        ));
    }

    #[test]
    fn test_icon_surrounded_via_decode() {
        let inlines = decode_para_inlines("before %[mdi:home] after", Format::Smd);
        assert_eq!(inlines.len(), 3);
        assert!(matches!(&inlines[0], Inline::Text(t) if t.value.as_str() == "before "));
        assert!(matches!(&inlines[1], Inline::Icon(..)));
        assert!(matches!(&inlines[2], Inline::Text(t) if t.value.as_str() == " after"));
    }

    #[test]
    fn test_icon_with_label_via_decode() {
        use stencila_codec::stencila_schema::Icon;

        let inlines = decode_para_inlines(r#"%[mdi:home]{label="Home"}"#, Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(
            &inlines[0],
            Inline::Icon(Icon { name, label: Some(label), decorative: None, .. })
                if name == "mdi:home" && label == "Home"
        ));
    }

    #[test]
    fn test_icon_with_combined_attrs_via_decode() {
        use stencila_codec::stencila_schema::Icon;

        let inlines =
            decode_para_inlines(r#"%[mdi:home]{label="Home" decorative=false}"#, Format::Smd);
        assert_eq!(inlines.len(), 1);
        assert!(matches!(
            &inlines[0],
            Inline::Icon(Icon { name, label: Some(label), decorative: Some(false), .. })
                if name == "mdi:home" && label == "Home"
        ));
    }

    // --- Icon serialization round-trip ---

    #[test]
    fn test_icon_round_trip_basic() {
        use stencila_codec::stencila_schema::Icon;

        // Decode → encode → decode should produce identical Icon
        let inlines1 = decode_para_inlines("%[mdi:home]", Format::Smd);
        assert_eq!(inlines1.len(), 1);
        assert!(matches!(&inlines1[0], Inline::Icon(..)));

        // Encode back to markdown
        use stencila_codec_markdown_trait::to_markdown;
        let md = to_markdown(&inlines1[0]);
        assert_eq!(md, "%[mdi:home]");

        // Decode again and compare
        let inlines2 = decode_para_inlines(&md, Format::Smd);
        assert_eq!(inlines2.len(), 1);
        assert!(matches!(
            &inlines2[0],
            Inline::Icon(Icon { name, .. }) if name == "mdi:home"
        ));
    }

    #[test]
    fn test_icon_round_trip_with_label() {
        let inlines1 = decode_para_inlines(r#"%[mdi:home]{label="Home"}"#, Format::Smd);
        assert_eq!(inlines1.len(), 1);

        // Extract the decoded Icon for comparison
        let icon1 = match &inlines1[0] {
            Inline::Icon(icon) => icon,
            other => panic!("Expected Inline::Icon, got: {other:?}"),
        };
        assert_eq!(icon1.name, "mdi:home");
        assert_eq!(icon1.label.as_deref(), Some("Home"));
        assert_eq!(icon1.decorative, None);

        // Encode back to markdown — must produce valid %[name]{attrs} syntax
        use stencila_codec_markdown_trait::to_markdown;
        let md = to_markdown(&inlines1[0]);
        assert!(
            md.starts_with("%[mdi:home]"),
            "Encoded markdown should start with icon syntax, got: {md}"
        );
        assert!(
            md.contains(r#"label="Home""#),
            "Encoded markdown should contain label attr, got: {md}"
        );

        // Decode again and verify semantic equality
        let inlines2 = decode_para_inlines(&md, Format::Smd);
        assert_eq!(inlines2.len(), 1);
        let icon2 = match &inlines2[0] {
            Inline::Icon(icon) => icon,
            other => panic!("Expected Inline::Icon after round-trip, got: {other:?}"),
        };
        assert_eq!(icon2.name, icon1.name);
        assert_eq!(icon2.label, icon1.label);
        assert_eq!(icon2.decorative, icon1.decorative);
    }

    #[test]
    fn test_icon_round_trip_with_combined_attrs() {
        let inlines1 =
            decode_para_inlines(r#"%[mdi:home]{label="Home" decorative=false}"#, Format::Smd);
        assert_eq!(inlines1.len(), 1);

        let icon1 = match &inlines1[0] {
            Inline::Icon(icon) => icon,
            other => panic!("Expected Inline::Icon, got: {other:?}"),
        };

        use stencila_codec_markdown_trait::to_markdown;
        let md = to_markdown(&inlines1[0]);

        let inlines2 = decode_para_inlines(&md, Format::Smd);
        assert_eq!(inlines2.len(), 1);
        let icon2 = match &inlines2[0] {
            Inline::Icon(icon) => icon,
            other => panic!("Expected Inline::Icon after round-trip, got: {other:?}"),
        };
        assert_eq!(icon2.name, icon1.name);
        assert_eq!(icon2.label, icon1.label);
        assert_eq!(icon2.decorative, icon1.decorative);
    }
}
