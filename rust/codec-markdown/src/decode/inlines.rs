use std::{collections::HashMap, ops::Range};

use codec_text_trait::to_text;
use markdown::{mdast, unist::Position};
use winnow::{
    ModalResult, Parser,
    ascii::{multispace0, space0},
    combinator::{alt, delimited, not, opt, peek, preceded, repeat, separated, terminated},
    stream::LocatingSlice as Located,
    token::{take, take_until, take_while},
};

use codec::{
    common::{indexmap::IndexMap, itertools::Itertools, tracing},
    format::Format,
    schema::{
        AudioObject, BooleanValidator, Button, Citation, CitationGroup, CitationMode,
        CodeExpression, CodeInline, Cord, DateTimeValidator, DateValidator, DurationValidator,
        Emphasis, EnumValidator, ImageObject, Inline, IntegerValidator, Link, MathInline, Node,
        NodeType, Note, NoteType, NumberValidator, Parameter, ParameterOptions, QuoteInline,
        Strikeout, StringValidator, Strong, StyledInline, Subscript, Superscript, Text,
        TimeValidator, TimestampValidator, Underline, Validator, VideoObject,
    },
};

use super::{
    Context,
    shared::{
        attrs, attrs_list, name, node_to_option_date, node_to_option_datetime,
        node_to_option_duration, node_to_option_i64, node_to_option_number, node_to_option_time,
        node_to_option_timestamp, node_to_string, take_until_unbalanced,
    },
};

pub(super) fn mds_to_inlines(mds: Vec<mdast::Node>, context: &mut Context) -> Vec<Inline> {
    // Collate all the inline nodes
    let mut nodes = Vec::new();
    for md in mds {
        if let mdast::Node::Text(mdast::Text { value, position }) = md {
            // Parse the text string for extensions not handled by the `markdown` crate e.g.
            // inline code, subscripts, superscripts etc and sentinel text like EDIT_END
            let mut inlines = inlines(&value)
                .into_iter()
                .map(|(inline, span)| {
                    let span = position
                        .as_ref()
                        .map(|position| {
                            (position.start.offset + span.start)..(position.start.offset + span.end)
                        })
                        .unwrap_or_default();
                    (inline, span)
                })
                .collect();
            nodes.append(&mut inlines);
        } else if let Some((inline, position)) = md_to_inline(md, context) {
            let span = position
                .map(|position| position.start.offset..position.end.offset)
                .unwrap_or_default();
            nodes.push((inline, span))
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
pub(super) fn inlines(input: &str) -> Vec<(Inline, Range<usize>)> {
    repeat(
        0..,
        alt((
            myst_role,
            qmd_inline_code,
            code_attrs,
            double_braces,
            math,
            citation_group,
            citation_parenthetical,
            citation_narrative,
            parameter,
            button,
            styled_inline,
            quote,
            strikeout,
            subscript,
            superscript,
            underline,
            html,
            string,
            character,
        ))
        .with_span(),
    )
    .parse(Located::new(input))
    .unwrap_or_else(|_| vec![(Inline::Text(Text::from(input)), 0..input.len())])
}

/// Parse a text string into a vector of `Inline` nodes
fn inlines_only(input: &str) -> Vec<Inline> {
    inlines(input)
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

// Parse a QMD inline expression
fn qmd_inline_code(input: &mut Located<&str>) -> ModalResult<Inline> {
    (
        delimited("`{", take_until(0.., '}'), '}'),
        delimited(multispace0, take_until(0.., '`'), '`'),
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

/// Parse math surrounded by one or two dollars into a `MathInline`.
fn math(input: &mut Located<&str>) -> ModalResult<Inline> {
    (alt((
        delimited("$$", take_until(0.., "$$"), "$$"),
        delimited("$", take_until(0.., "$"), "$"),
    )))
    .map(|code: &str| {
        Inline::MathInline(MathInline {
            code: code.into(),
            math_language: Some("tex".into()),
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
            // Only match single tilde, because doubles are for `Strikeout`
            ('~', peek(not('~'))),
            take_until(1.., '~'),
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
        delimited('^', take_until(0.., '^'), '^'),
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
    const CHARS: &str = "~@#$^&[]{`<>";
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
    use codec::schema::ExecutionMode;
    use common_dev::pretty_assertions::assert_eq;

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
        let is = inlines(&Located::new("`${a} ${b}`"));
        assert_eq!(is.len(), 1);
        assert_eq!(is[0].0.node_type(), NodeType::CodeInline);
    }

    #[test]
    fn test_math() {
        assert_eq!(
            math(&mut Located::new("$\\pi r^2$")).unwrap(),
            Inline::MathInline(MathInline {
                code: "\\pi r^2".into(),
                math_language: Some("tex".into()),
                ..Default::default()
            })
        );

        assert_eq!(
            math(&mut Located::new("$$\\pi r^2$$")).unwrap(),
            Inline::MathInline(MathInline {
                code: "\\pi r^2".into(),
                math_language: Some("tex".into()),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_underline() {
        underline(&mut Located::new("<u></u>")).unwrap();
        underline(&mut Located::new("<u>underlined</u>")).unwrap();

        let inlines = inlines("before <u>underlined</u> after");
        assert_eq!(inlines.len(), 3);
        assert!(matches!(inlines[1].0, Inline::Underline(..)));
    }
}
