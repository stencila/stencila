use std::{collections::HashMap, ops::Range};

use markdown::{mdast, unist::Position};
use winnow::{
    ascii::{multispace0, multispace1, space0},
    combinator::{alt, delimited, not, opt, peek, preceded, repeat, separated, terminated},
    stream::{Located, Stream},
    token::{take, take_until, take_while},
    PResult, Parser,
};

use codec::{
    common::{indexmap::IndexMap, itertools::Itertools},
    format::Format,
    schema::{
        AudioObject, BooleanValidator, Button, Cite, CiteGroup, CodeExpression, CodeInline, Cord,
        DateTimeValidator, DateValidator, DeleteInline, DurationValidator, Emphasis, EnumValidator,
        ImageObject, Inline, InsertInline, InstructionInline, InstructionInlineOptions,
        InstructionMessage, IntegerValidator, Link, MathInline, ModifyInline, Node, Note, NoteType,
        NumberValidator, Parameter, ParameterOptions, QuoteInline, ReplaceInline, Strikeout,
        StringValidator, Strong, StyledInline, Subscript, SuggestionInlineType, Superscript, Text,
        TimeValidator, TimestampValidator, Underline, Validator, VideoObject,
    },
};

use super::{
    shared::{
        assignee, attrs, name, node_to_from_str, node_to_option_date, node_to_option_datetime,
        node_to_option_duration, node_to_option_i64, node_to_option_number, node_to_option_time,
        node_to_option_timestamp, node_to_string, take_until_unbalanced,
    },
    Context,
};

const EDIT_START: &str = "[[";
const EDIT_WITH: &str = ">>";
const EDIT_END: &str = "]]";

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
    let mut boundaries = Vec::new();
    for (inline, span) in nodes {
        if let Inline::Text(text) = &inline {
            // Note: currently, mainly for performance reasons, we do not add mapping entries
            // for `Inline::Text` nodes.
            if text.value.as_str() == EDIT_WITH {
                // A `>>` separator so associated inlines since last boundary with the previous
                // `ReplaceInline` or `ModifyInline`
                if let Some(boundary) = boundaries.pop() {
                    let children = inlines.drain(boundary..).collect();
                    match inlines.last_mut() {
                        Some(
                            Inline::ReplaceInline(ReplaceInline { content, .. })
                            | Inline::ModifyInline(ModifyInline { content, .. }),
                        ) => {
                            *content = children;
                            boundaries.push(inlines.len());
                        }

                        _ => {
                            // This should not happen, but if it does push the separator
                            inlines.push(inline);
                        }
                    }
                } else {
                    // A `>>` fragment that is not a separator, so just push
                    inlines.push(inline);
                }
            } else if text.value.as_str() == EDIT_END {
                // A `]]` terminator so associate inlines since last boundary with the previous
                // `InstructionInline`, `InsertInline`, `DeleteInline`, etc and map end
                if let Some(boundary) = boundaries.pop() {
                    // End the mapping for the previous inline
                    context.map_end(span.end);

                    let children = inlines.drain(boundary..).collect();
                    match inlines.last_mut() {
                        Some(
                            Inline::InstructionInline(InstructionInline {
                                content: Some(content),
                                ..
                            })
                            | Inline::InsertInline(InsertInline { content, .. })
                            | Inline::DeleteInline(DeleteInline { content, .. }),
                        ) => {
                            *content = children;
                        }

                        Some(Inline::ReplaceInline(ReplaceInline { replacement, .. })) => {
                            *replacement = children;
                        }

                        Some(Inline::ModifyInline(..)) => {
                            // Ignore "simulated" replacement content
                        }

                        _ => {
                            // This should not happen, but if it does push the terminator
                            inlines.push(inline);
                        }
                    }

                    // If the inline before this one was an instruction then associate the two.
                    // Also extend the range of the mapping for the instruction to the end of
                    // the suggestion.
                    if matches!(
                        inlines.iter().rev().nth(1),
                        Some(Inline::InstructionInline(..))
                    ) {
                        let (node_id, suggestion) = match inlines.pop() {
                            Some(Inline::InsertInline(inline)) => {
                                (inline.node_id(), SuggestionInlineType::InsertInline(inline))
                            }
                            Some(Inline::DeleteInline(inline)) => {
                                (inline.node_id(), SuggestionInlineType::DeleteInline(inline))
                            }
                            Some(Inline::ReplaceInline(inline)) => (
                                inline.node_id(),
                                SuggestionInlineType::ReplaceInline(inline),
                            ),
                            Some(Inline::ModifyInline(inline)) => {
                                (inline.node_id(), SuggestionInlineType::ModifyInline(inline))
                            }
                            _ => unreachable!(),
                        };
                        if let Some(Inline::InstructionInline(instruct)) = inlines.last_mut() {
                            // Associate the suggestion with the instruction
                            instruct.options.suggestion = Some(suggestion);

                            // Extend the instruction to the end of the suggestion
                            context.map_extend(instruct.node_id(), node_id);
                        }
                    }
                } else {
                    // A `]]` fragment that is not a terminator, so just push
                    inlines.push(inline);
                }
            } else if let Some(Inline::Text(previous_text)) = inlines.last_mut() {
                // The previous inline was text so merge the two
                previous_text.value.push_str(&text.value);
            } else {
                // Just a plain text node so just map and push
                inlines.push(inline);
            }
        } else if matches!(
            inline,
            Inline::InstructionInline(InstructionInline {
                content: Some(..),
                ..
            }) | Inline::InsertInline(..)
                | Inline::DeleteInline(..)
                | Inline::ReplaceInline(..)
                | Inline::ModifyInline(..)
        ) {
            // An inline that registers a boundary
            if let Some(node_id) = inline.node_id() {
                context.map_start(span.start, inline.node_type(), node_id)
            }
            inlines.push(inline);
            boundaries.push(inlines.len());
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
                math_language: Some("tex".to_string()),
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

            let inline = if let Ok(format) = Format::from_url(&content_url) {
                if format.is_audio() {
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
                }
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
            code_attrs,
            double_braces,
            cite_group,
            cite,
            parameter,
            button,
            styled_inline,
            quote,
            strikeout,
            subscript,
            superscript,
            underline,
            instruction_inline,
            insert_inline,
            delete_inline,
            replace_inline,
            modify_inline,
            edit_with,
            edit_end,
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

/// Parse inline code with optional attributes in curly braces e.g. `\`code\`{attr1 attr2}`
/// into a `CodeFragment`, `CodeExpression` or `MathFragment` node.
///
/// The `attrs` are optional because plain `CodeFragment`s also end up being
/// passed to this function
fn code_attrs(input: &mut Located<&str>) -> PResult<Inline> {
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
        let mut auto_exec = None;

        for (name, value) in options {
            if name == "exec" {
                exec = true
            } else if lang.is_none() && value.is_none() {
                lang = Some(name.to_string());
            } else if name == "auto" {
                if let Some(value) = value {
                    auto_exec = node_to_string(value).parse().ok()
                }
            }
        }

        if exec {
            Inline::CodeExpression(CodeExpression {
                code: code.into(),
                programming_language: lang,
                auto_exec,
                ..Default::default()
            })
        } else if matches!(
            lang.as_deref(),
            Some("asciimath") | Some("mathml") | Some("latex") | Some("tex")
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
/// This supports the Jupyter "Python Markdown" extension syntax for
/// interpolated variables / expressions: `{{x}}`
///
/// Does not support the single curly brace syntax (as in Python, Rust and JSX) i.e. `{x}`
/// given that is less specific and could conflict with other user content.
///
/// Does not support JavaScript style "dollared-brace" syntax i.e. `${x}` since some
/// at least some Markdown parsers seem to parse that as TeX math (even though there
/// is no closing brace).
///
/// The language of the code expression can be added in a curly brace suffix.
/// e.g. `{{2 * 2}}{r}` is equivalent to `\`r 2 * 2\``{r exec} in Markdown or to
/// `\`r 2 * 2\` in R Markdown.
fn double_braces(input: &mut Located<&str>) -> PResult<Inline> {
    (delimited("{{", take_until(0.., "}}"), "}}"), opt(attrs))
        .map(|(code, options)| {
            let mut options: IndexMap<&str, _> = options.unwrap_or_default().into_iter().collect();

            Inline::CodeExpression(CodeExpression {
                code: code.into(),
                programming_language: options.first().map(|(lang, ..)| lang.to_string()),
                auto_exec: options
                    .swap_remove("auto")
                    .flatten()
                    .and_then(node_to_from_str),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a string into a narrative `Cite` node
///
/// This attempts to follow Pandoc's citation handling as closely as possible
/// (see <https://pandoc.org/MANUAL.html#citations>).
///
/// The following properties of a `Cite` are parsed:
///   - [x] target
///   - [ ] citation_mode
///   - [ ] page_start
///   - [ ] page_end
///   - [ ] pagination
///   - [ ] citation_prefix
///   - [ ] citation_suffix
///   - [ ] citation_intent
fn cite(input: &mut Located<&str>) -> PResult<Inline> {
    // TODO: Parse more properties of citations
    preceded('@', take_while(1.., |chr: char| chr.is_alphanumeric()))
        .map(|target: &str| {
            Inline::Cite(Cite {
                target: target.into(),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a string into a `CiteGroup` node or parenthetical `Cite` node.
///
/// If there is only one citation within square brackets then a parenthetical `Cite` node is
/// returned. Otherwise, the `Cite` nodes are grouped into into a `CiteGroup`.
fn cite_group(input: &mut Located<&str>) -> PResult<Inline> {
    let cite =
        preceded('@', take_while(1.., |chr: char| chr.is_alphanumeric())).map(|res: &str| {
            let target = res.into();
            Inline::Cite(Cite {
                target,
                ..Default::default()
            })
        });

    delimited(
        '[',
        separated(1.., cite, (multispace0, ';', multispace0)),
        ']',
    )
    .map(|items: Vec<Inline>| {
        if items.len() == 1 {
            items[0].clone()
        } else {
            Inline::CiteGroup(CiteGroup {
                items: items
                    .iter()
                    .filter_map(|item| match item {
                        Inline::Cite(cite) => Some(cite),
                        _ => None,
                    })
                    .cloned()
                    .collect(),
                ..Default::default()
            })
        }
    })
    .parse_next(input)
}

/// Parse a `Parameter`.
fn parameter(input: &mut Located<&str>) -> PResult<Inline> {
    (delimited("&[", name, ']'), opt(attrs))
        .map(|(name, attrs)| {
            let attrs = attrs.unwrap_or_default();
            let first = attrs
                .first()
                .map(|(name, ..)| Some(Node::String(name.to_string())));

            let mut options: HashMap<&str, Option<Node>> = attrs.into_iter().collect();

            let typ = options
                .remove("type")
                .or(first.clone())
                .flatten()
                .map(node_to_string);
            let typ = typ.as_deref();

            let label = options.remove("label").flatten().map(node_to_string);

            let validator = if matches!(typ, Some("boolean")) || matches!(typ, Some("bool")) {
                Some(Validator::BooleanValidator(BooleanValidator::default()))
            } else if matches!(typ, Some("enum")) {
                let values = options.remove("vals").flatten();
                let values = match values {
                    Some(node) => match node {
                        // Usually the supplied node is an array, which we need to convert
                        // to a vector of `Node`s
                        Node::Array(array) => array
                            .iter()
                            .map(|primitive| primitive.clone().into())
                            .collect(),
                        _ => vec![node],
                    },
                    None => vec![],
                };
                Some(Validator::EnumValidator(EnumValidator {
                    values,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("integer")) || matches!(typ, Some("int")) {
                let minimum = options
                    .remove("min")
                    .flatten()
                    .and_then(node_to_option_number);
                let exclusive_minimum = options
                    .remove("emin")
                    .flatten()
                    .and_then(node_to_option_number);
                let maximum = options
                    .remove("max")
                    .flatten()
                    .and_then(node_to_option_number);
                let exclusive_maximum = options
                    .remove("emax")
                    .flatten()
                    .and_then(node_to_option_number);
                let multiple_of = options
                    .remove("mult")
                    .or_else(|| options.remove("step"))
                    .flatten()
                    .and_then(node_to_option_number);
                Some(Validator::IntegerValidator(IntegerValidator {
                    minimum,
                    exclusive_minimum,
                    maximum,
                    exclusive_maximum,
                    multiple_of,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("number")) || matches!(typ, Some("num")) {
                let minimum = options
                    .remove("min")
                    .flatten()
                    .and_then(node_to_option_number);
                let exclusive_minimum = options
                    .remove("emin")
                    .flatten()
                    .and_then(node_to_option_number);
                let maximum = options
                    .remove("max")
                    .flatten()
                    .and_then(node_to_option_number);
                let exclusive_maximum = options
                    .remove("emax")
                    .flatten()
                    .and_then(node_to_option_number);
                let multiple_of = options
                    .remove("mult")
                    .or_else(|| options.remove("step"))
                    .flatten()
                    .and_then(node_to_option_number);
                Some(Validator::NumberValidator(NumberValidator {
                    minimum,
                    exclusive_minimum,
                    maximum,
                    exclusive_maximum,
                    multiple_of,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("string")) || matches!(typ, Some("str")) {
                let min_length = options
                    .remove("minlength")
                    .or_else(|| options.remove("min"))
                    .flatten()
                    .and_then(node_to_option_i64);
                let max_length = options
                    .remove("maxlength")
                    .or_else(|| options.remove("max"))
                    .flatten()
                    .and_then(node_to_option_i64);
                let pattern = options
                    .remove("pattern")
                    .or_else(|| options.remove("regex"))
                    .flatten()
                    .map(node_to_string);
                Some(Validator::StringValidator(StringValidator {
                    min_length,
                    max_length,
                    pattern,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("date")) {
                let minimum = options
                    .remove("min")
                    .flatten()
                    .and_then(node_to_option_date);
                let maximum = options
                    .remove("max")
                    .flatten()
                    .and_then(node_to_option_date);
                Some(Validator::DateValidator(DateValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("time")) {
                let minimum = options
                    .remove("min")
                    .flatten()
                    .and_then(node_to_option_time);
                let maximum = options
                    .remove("max")
                    .flatten()
                    .and_then(node_to_option_time);
                Some(Validator::TimeValidator(TimeValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("datetime")) {
                let minimum = options
                    .remove("min")
                    .flatten()
                    .and_then(node_to_option_datetime);
                let maximum = options
                    .remove("max")
                    .flatten()
                    .and_then(node_to_option_datetime);
                Some(Validator::DateTimeValidator(DateTimeValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("timestamp")) {
                let minimum = options
                    .remove("min")
                    .flatten()
                    .and_then(node_to_option_timestamp);
                let maximum = options
                    .remove("max")
                    .flatten()
                    .and_then(node_to_option_timestamp);
                Some(Validator::TimestampValidator(TimestampValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else if matches!(typ, Some("duration")) {
                let minimum = options
                    .remove("min")
                    .flatten()
                    .and_then(node_to_option_duration);
                let maximum = options
                    .remove("max")
                    .flatten()
                    .and_then(node_to_option_duration);
                Some(Validator::DurationValidator(DurationValidator {
                    minimum,
                    maximum,
                    ..Default::default()
                }))
            } else {
                None
            };

            let default = options
                .remove("default")
                .or_else(|| options.remove("def"))
                .flatten()
                .map(Box::new);

            let value = options
                .remove("value")
                .or_else(|| options.remove("val"))
                .flatten()
                .map(Box::new);

            Inline::Parameter(Parameter {
                name: name.into(),
                value,
                options: Box::new(ParameterOptions {
                    label,
                    validator,
                    default,
                    ..Default::default()
                }),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a `Button`
fn button(input: &mut Located<&str>) -> PResult<Inline> {
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
fn styled_inline(input: &mut Located<&str>) -> PResult<Inline> {
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
fn strikeout(input: &mut Located<&str>) -> PResult<Inline> {
    delimited("~~", take_until(0.., "~~"), "~~")
        .map(|content: &str| Inline::Strikeout(Strikeout::new(inlines_only(content))))
        .parse_next(input)
}

/// Parse a string into a `Subscript` node
fn subscript(input: &mut Located<&str>) -> PResult<Inline> {
    delimited(
        // Only match single tilde, because doubles are for `Strikeout`
        ('~', peek(not('~'))),
        take_until(1.., '~'),
        '~',
    )
    .map(|content: &str| Inline::Subscript(Subscript::new(inlines_only(content))))
    .parse_next(input)
}

/// Parse a string into a `Superscript` node
fn superscript(input: &mut Located<&str>) -> PResult<Inline> {
    delimited('^', take_until(0.., '^'), '^')
        .map(|content: &str| Inline::Superscript(Superscript::new(inlines_only(content))))
        .parse_next(input)
}

/// Parse <q> tags into a `QuoteInline` node
fn quote(input: &mut Located<&str>) -> PResult<Inline> {
    delimited("<q>", take_until(0.., "</q>"), "</q>")
        .map(|content: &str| Inline::QuoteInline(QuoteInline::new(inlines_only(content))))
        .parse_next(input)
}

/// Parse <u> tags into a `Underline` node
fn underline(input: &mut Located<&str>) -> PResult<Inline> {
    delimited("<u>", take_until(0.., "</u>"), "</u>")
        .map(|content: &str| Inline::Underline(Underline::new(inlines_only(content))))
        .parse_next(input)
}

/// Parse a string into a `InstructionInline` node
fn instruction_inline(input: &mut Located<&str>) -> PResult<Inline> {
    preceded(
        terminated("[[do", multispace0),
        (opt(delimited('@', assignee, multispace1)), take_until_edit),
    )
    .map(|(assignee, (text, term))| {
        Inline::InstructionInline(InstructionInline {
            messages: vec![InstructionMessage::from(text.trim())],
            content: (term == EDIT_WITH).then_some(Vec::new()),
            options: Box::new(InstructionInlineOptions {
                assignee: assignee.map(|handle| handle.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Take characters until `EDIT_WITH` or `EDIT_END`
fn take_until_edit<'s>(input: &mut Located<&'s str>) -> PResult<(&'s str, &'static str)> {
    let mut last = ' ';
    for (index, char) in input.char_indices() {
        if (last == '>' && char == '>') || last == ']' && char == ']' {
            let res = input.next_slice(index - 1);
            input.next_token().map(|_| input.next_token());
            return Ok((res, if char == '>' { EDIT_WITH } else { EDIT_END }));
        }

        last = char;
    }
    Ok((input.next_slice(input.len()), EDIT_END))
}

/// Parse a string into a `InsertInline` node
fn insert_inline(input: &mut Located<&str>) -> PResult<Inline> {
    (EDIT_START, alt(("insert", "ins")), ' ')
        .map(|_| Inline::InsertInline(InsertInline::default()))
        .parse_next(input)
}

/// Parse a string into a `DeleteInline` node
fn delete_inline(input: &mut Located<&str>) -> PResult<Inline> {
    (EDIT_START, alt(("delete", "del")), ' ')
        .map(|_| Inline::DeleteInline(DeleteInline::default()))
        .parse_next(input)
}

/// Parse a string into a `ReplaceInline` node
fn replace_inline(input: &mut Located<&str>) -> PResult<Inline> {
    (EDIT_START, alt(("replace", "rep")), ' ')
        .map(|_| Inline::ReplaceInline(ReplaceInline::default()))
        .parse_next(input)
}

/// Parse a string into a `ModifyInline` node
fn modify_inline(input: &mut Located<&str>) -> PResult<Inline> {
    (EDIT_START, alt(("modify", "mod")), ' ')
        .map(|_| Inline::ModifyInline(ModifyInline::default()))
        .parse_next(input)
}

/// Parse a `with:` word indicating the replacement content for a `ReplaceInline` or `ModifyInline` node
fn edit_with(input: &mut Located<&str>) -> PResult<Inline> {
    EDIT_WITH
        .map(|_| Inline::Text(Text::from(EDIT_WITH)))
        .parse_next(input)
}

/// Parse double closing square brackets `]]` indicating the end of content
/// for an edit node
fn edit_end(input: &mut Located<&str>) -> PResult<Inline> {
    EDIT_END
        .map(|_| Inline::Text(Text::from(EDIT_END)))
        .parse_next(input)
}

/// Accumulate characters into a `Text` node
///
/// Will greedily take as many characters as possible, excluding those that appear at the
/// start of other inline parsers e.g. '$', '['
fn string(input: &mut Located<&str>) -> PResult<Inline> {
    const CHARS: &str = "~@#$^&[]{`<>";
    take_while(1.., |chr: char| !CHARS.contains(chr))
        .map(|val: &str| Inline::Text(Text::new(val.into())))
        .parse_next(input)
}

/// Take a single character into a `Text` node
///
/// Necessary so that the characters not consumed by `string` are not lost.
fn character(input: &mut Located<&str>) -> PResult<Inline> {
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
                ["[", alt, "](", url, ")"].concat()
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
mod tests {
    use codec::schema::AutomaticExecution;
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_code_attrs() {
        code_attrs(&mut Located::new("``")).unwrap();
        code_attrs(&mut Located::new("``{}")).unwrap();
        code_attrs(&mut Located::new("`a + b`{}")).unwrap();
        code_attrs(&mut Located::new("`a + b`{python}")).unwrap();
        code_attrs(&mut Located::new("`a + b`{python exec}")).unwrap();
        code_attrs(&mut Located::new("`a + b`{python exec auto=always}")).unwrap();

        assert_eq!(
            code_attrs(&mut Located::new("`a + b`{javascript exec auto=never}")).unwrap(),
            Inline::CodeExpression(CodeExpression {
                code: "a + b".into(),
                programming_language: Some("javascript".into()),
                auto_exec: Some(AutomaticExecution::Never),
                ..Default::default()
            })
        );

        assert!(code_attrs(&mut Located::new("=`1*1`")).is_err());
        assert!(code_attrs(&mut Located::new("= `2+2`")).is_err());
    }

    #[test]
    fn test_underline() {
        underline(&mut Located::new("<u></u>")).unwrap();
        underline(&mut Located::new("<u>underlined</u>")).unwrap();

        let inlines = inlines("before <u>underlined</u> after");
        assert_eq!(inlines.len(), 3);
        assert!(matches!(inlines[1].0, Inline::Underline(..)));
    }

    #[test]
    fn test_instruction_inline() {
        instruction_inline(&mut Located::new("[[do something]]")).unwrap();

        let ins = inlines("before [[do something]] after");
        assert_eq!(ins.len(), 3);
        assert_eq!(ins[0].0, Inline::Text(Text::from("before ")));
        assert!(matches!(ins[1].0, Inline::InstructionInline(..)));
        assert_eq!(ins[2].0, Inline::Text(Text::from(" after")));

        let ins = inlines("before [[do something >> this]] after");
        assert_eq!(ins.len(), 5);
        assert_eq!(ins[0].0, Inline::Text(Text::from("before ")));
        assert!(matches!(ins[1].0, Inline::InstructionInline(..)));
        assert_eq!(ins[2].0, Inline::Text(Text::from(" this")));
        assert_eq!(ins[3].0, Inline::Text(Text::from("]]")));
        assert_eq!(ins[4].0, Inline::Text(Text::from(" after")));
    }
}
