use std::collections::HashMap;

use markdown::{mdast, unist::Position};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until, take_while1},
    character::complete::{anychar, char, multispace0, multispace1},
    combinator::{map, not, opt, peek},
    multi::{many0, many_till, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use codec::{
    common::indexmap::IndexMap,
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
        assignee, attrs, name, node_to_option_date, node_to_option_datetime,
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
            // Parse the string for extensions not handled by the `markdown` crate e.g.
            // inline code, subscripts, superscripts etc and sentinel text like EDIT_END
            let mut parsed = parse_inlines(&value, position);
            nodes.append(&mut parsed);
        } else if let Some(inline) = md_to_inline(md, context) {
            nodes.push(inline)
        }
    }

    // Iterate over the inlines
    let mut inlines = Vec::with_capacity(nodes.len());
    let mut boundaries = Vec::new();
    for node in nodes {
        if let Inline::Text(text) = &node {
            if text.value.as_str() == EDIT_WITH {
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

                        _ => inlines.push(node),
                    }
                } else {
                    inlines.push(node)
                }
            } else if text.value.as_str() == EDIT_END {
                if let Some(boundary) = boundaries.pop() {
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

                        _ => inlines.push(node),
                    }

                    // If the the inline before this one was an instruction then associate the two
                    if matches!(
                        inlines.iter().rev().nth(1),
                        Some(Inline::InstructionInline(..))
                    ) {
                        let suggestion = match inlines.pop() {
                            Some(Inline::InsertInline(block)) => {
                                SuggestionInlineType::InsertInline(block)
                            }
                            Some(Inline::DeleteInline(block)) => {
                                SuggestionInlineType::DeleteInline(block)
                            }
                            Some(Inline::ReplaceInline(block)) => {
                                SuggestionInlineType::ReplaceInline(block)
                            }
                            Some(Inline::ModifyInline(block)) => {
                                SuggestionInlineType::ModifyInline(block)
                            }
                            _ => unreachable!(),
                        };
                        if let Some(Inline::InstructionInline(instruct)) = inlines.last_mut() {
                            instruct.options.suggestion = Some(suggestion);
                        }
                    }
                } else {
                    inlines.push(node)
                }
            } else {
                if let Some(Inline::Text(previous_text)) = inlines.last_mut() {
                    previous_text.value.push_str(&text.value);
                } else {
                    inlines.push(node)
                }
            }
        } else if matches!(
            node,
            Inline::InstructionInline(InstructionInline {
                content: Some(..),
                ..
            }) | Inline::InsertInline(..)
                | Inline::DeleteInline(..)
                | Inline::ReplaceInline(..)
                | Inline::ModifyInline(..)
        ) {
            inlines.push(node);
            boundaries.push(inlines.len());
        } else {
            inlines.push(node)
        }
    }

    inlines
}

/// Transform MDAST inline nodes to Stencila inlines nodes
fn md_to_inline(md: mdast::Node, context: &mut Context) -> Option<Inline> {
    Some(match md {
        mdast::Node::Delete(mdast::Delete { children, position }) => {
            let node = Strikeout::new(mds_to_inlines(children, context));
            context.map(position, node.node_type(), node.node_id());
            Inline::Strikeout(node)
        }

        mdast::Node::Emphasis(mdast::Emphasis { children, position }) => {
            let node = Emphasis::new(mds_to_inlines(children, context));
            context.map(position, node.node_type(), node.node_id());
            Inline::Emphasis(node)
        }

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
            context.map(position, node.node_type(), node.node_id());
            Inline::Note(node)
        }

        mdast::Node::InlineCode(mdast::InlineCode { value, position }) => {
            let node = CodeInline::new(value.into());
            context.map(position, node.node_type(), node.node_id());
            Inline::CodeInline(node)
        }

        mdast::Node::InlineMath(mdast::InlineMath { value, position }) => {
            let node = MathInline {
                code: value.into(),
                math_language: Some("tex".to_string()),
                ..Default::default()
            };
            context.map(position, node.node_type(), node.node_id());
            Inline::MathInline(node)
        }

        mdast::Node::Image(mdast::Image {
            url: content_url,
            alt,
            title,
            position,
        }) => {
            let title = title.map(|title| vec![Inline::Text(Text::from(title))]);
            let caption = (!alt.is_empty()).then_some(vec![Inline::Text(Text::from(alt))]);

            if let Ok(format) = Format::from_url(&content_url) {
                if format.is_audio() {
                    let node = AudioObject {
                        content_url,
                        caption,
                        title,
                        ..Default::default()
                    };
                    context.map(position, node.node_type(), node.node_id());
                    Inline::AudioObject(node)
                } else if format.is_video() {
                    let node = VideoObject {
                        content_url,
                        caption,
                        title,
                        ..Default::default()
                    };
                    context.map(position, node.node_type(), node.node_id());
                    Inline::VideoObject(node)
                } else {
                    let node = ImageObject {
                        content_url,
                        caption,
                        title,
                        ..Default::default()
                    };
                    context.map(position, node.node_type(), node.node_id());
                    Inline::ImageObject(node)
                }
            } else {
                let node = ImageObject {
                    content_url,
                    caption,
                    title,
                    ..Default::default()
                };
                context.map(position, node.node_type(), node.node_id());
                Inline::ImageObject(node)
            }
        }

        mdast::Node::Link(mdast::Link {
            children,
            url,
            title,
            position,
        }) => {
            let node = Link {
                target: url,
                title,
                content: mds_to_inlines(children, context),
                ..Default::default()
            };
            context.map(position, node.node_type(), node.node_id());
            Inline::Link(node)
        }

        mdast::Node::Strong(mdast::Strong { children, position }) => {
            let node = Strong::new(mds_to_inlines(children, context));
            context.map(position, node.node_type(), node.node_id());
            Inline::Strong(node)
        }

        mdast::Node::Text(mdast::Text { value, position }) => {
            // This should not be reach because plain text nodes are handled elsewhere
            // but it case it is, return it so not lost
            let node = Text::from(value);
            context.map(position, node.node_type(), node.node_id());
            Inline::Text(node)
        }

        _ => {
            // TODO: Any unexpected blocks should be decomposed to their inline children
            context.lost("Inline");
            return None;
        }
    })
}

/// Parse a text string into a vector of `Inline` nodes
pub(super) fn parse_inlines(input: &str, _position: Option<Position>) -> Vec<Inline> {
    many0(alt((
        button,
        code_attrs,
        double_braces,
        cite_group,
        cite,
        parameter,
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
    )))(input)
    .map_or_else(
        |_| vec![Inline::Text(Text::from(input))],
        |(.., inlines)| inlines,
    )
}

/// Parse inline code with optional attributes in curly braces e.g. `\`code\`{attr1 attr2}`
/// into a `CodeFragment`, `CodeExpression` or `MathFragment` node.
///
/// The `attrs` are optional because plain `CodeFragment`s also end up being
/// passed to this function
fn code_attrs(input: &str) -> IResult<&str, Inline> {
    map(
        preceded(
            not(peek(pair(char('='), multispace0))),
            pair(delimited(char('`'), take_until("`"), char('`')), opt(attrs)),
        ),
        |(code, options)| {
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
                } else if value.is_none() {
                    lang = Some(name);
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
        },
    )(input)
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
fn double_braces(input: &str) -> IResult<&str, Inline> {
    map(
        pair(
            delimited(tag("{{"), take_until("}}"), tag("}}")),
            opt(delimited(char('{'), take_until("}"), char('}'))),
        ),
        |(code, options): (&str, Option<&str>)| {
            let code = code.into();
            let lang = match options {
                Some(attrs) => {
                    let attrs = attrs.split_whitespace().collect::<Vec<&str>>();
                    attrs.first().map(|item| item.to_string())
                }
                None => None,
            };
            Inline::CodeExpression(CodeExpression {
                code,
                programming_language: lang,
                ..Default::default()
            })
        },
    )(input)
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
fn cite(input: &str) -> IResult<&str, Inline> {
    // TODO: Parse more properties of citations
    map(
        preceded(char('@'), take_while1(|chr: char| chr.is_alphanumeric())),
        |res: &str| {
            let target = res.into();
            Inline::Cite(Cite {
                target,
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a string into a `CiteGroup` node or parenthetical `Cite` node.
///
/// If there is only one citation within square brackets then a parenthetical `Cite` node is
/// returned. Otherwise, the `Cite` nodes are grouped into into a `CiteGroup`.
fn cite_group(input: &str) -> IResult<&str, Inline> {
    let cite = map(
        preceded(char('@'), take_while1(|chr: char| chr.is_alphanumeric())),
        |res: &str| {
            let target = res.into();
            Inline::Cite(Cite {
                target,
                ..Default::default()
            })
        },
    );

    map(
        delimited(
            char('['),
            separated_list1(tuple((multispace0, tag(";"), multispace0)), cite),
            char(']'),
        ),
        |items: Vec<Inline>| {
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
        },
    )(input)
}

/// Parse a `Parameter`.
fn parameter(input: &str) -> IResult<&str, Inline> {
    map(
        pair(delimited(tag("&["), name, char(']')), opt(attrs)),
        |(name, attrs)| {
            let attrs = attrs.unwrap_or_default();
            let first = attrs
                .first()
                .map(|(name, ..)| Some(Node::String(name.clone())));
            let mut options: HashMap<String, Option<Node>> = attrs.into_iter().collect();

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
        },
    )(input)
}

/// Parse a `Button`
fn button(input: &str) -> IResult<&str, Inline> {
    map(
        tuple((
            delimited(tag("#["), take_until("]"), char(']')),
            opt(delimited(char('`'), take_until("`"), char('`'))),
            opt(attrs),
        )),
        |(name, condition, options)| {
            let mut options: IndexMap<String, Option<Node>> =
                options.unwrap_or_default().into_iter().collect();

            let programming_language = if let Some((lang, None)) = options.first() {
                Some(lang.clone())
            } else {
                None
            };

            let code = condition.map_or_else(Cord::default, Cord::from);

            let label = options.swap_remove("label").flatten().map(node_to_string);

            Inline::Button(Button {
                name: name.to_string(),
                programming_language,
                code,
                label,
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a [`StyledInline`].
fn styled_inline(input: &str) -> IResult<&str, Inline> {
    map(
        tuple((
            delimited(char('['), take_until_unbalanced('[', ']'), char(']')),
            delimited(char('{'), take_until_unbalanced('{', '}'), char('}')),
        )),
        |(content, code): (&str, &str)| {
            Inline::StyledInline(StyledInline {
                content: parse_inlines(content, None),
                code: code.into(),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a string into a `Strikeout` node
fn strikeout(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(tag("~~"), take_until("~~"), tag("~~")),
        |content: &str| Inline::Strikeout(Strikeout::new(parse_inlines(content, None))),
    )(input)
}

/// Parse a string into a `Subscript` node
fn subscript(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(
            // Only match single tilde, because doubles are for `Strikeout`
            tuple((char('~'), peek(not(char('~'))))),
            take_until("~"),
            char('~'),
        ),
        |content: &str| Inline::Subscript(Subscript::new(parse_inlines(content, None))),
    )(input)
}

/// Parse a string into a `Superscript` node
fn superscript(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(char('^'), take_until("^"), char('^')),
        |content: &str| Inline::Superscript(Superscript::new(parse_inlines(content, None))),
    )(input)
}

/// Parse <q> tags into a `QuoteInline` node
fn quote(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(tag("<q>"), take_until("</q>"), tag("</q>")),
        |content: &str| Inline::QuoteInline(QuoteInline::new(parse_inlines(content, None))),
    )(input)
}

/// Parse <u> tags into a `Underline` node
fn underline(input: &str) -> IResult<&str, Inline> {
    map(
        delimited(tag("<u>"), take_until("</u>"), tag("</u>")),
        |content: &str| Inline::Underline(Underline::new(parse_inlines(content, None))),
    )(input)
}

/// Parse a string into a `InstructionInline` node
fn instruction_inline(input: &str) -> IResult<&str, Inline> {
    map(
        preceded(
            terminated(tag("[[do"), multispace0),
            tuple((
                opt(delimited(char('@'), assignee, multispace1)),
                map(
                    many_till(anychar, peek(alt((tag(EDIT_WITH), tag(EDIT_END))))),
                    |(chars, ..)| -> String { chars.iter().collect() },
                ),
                alt((tag(EDIT_WITH), tag(EDIT_END))),
            )),
        ),
        |(assignee, text, term)| {
            Inline::InstructionInline(InstructionInline {
                messages: vec![InstructionMessage::from(text.trim())],
                content: (term == EDIT_WITH).then_some(Vec::new()),
                options: Box::new(InstructionInlineOptions {
                    assignee: assignee.map(|handle| handle.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
        },
    )(input)
}

/// Parse a string into a `InsertInline` node
fn insert_inline(input: &str) -> IResult<&str, Inline> {
    map(
        tuple((tag(EDIT_START), alt((tag("insert"), tag("ins"))), tag(" "))),
        |_| Inline::InsertInline(InsertInline::default()),
    )(input)
}

/// Parse a string into a `DeleteInline` node
fn delete_inline(input: &str) -> IResult<&str, Inline> {
    map(
        tuple((tag(EDIT_START), alt((tag("delete"), tag("del"))), tag(" "))),
        |_| Inline::DeleteInline(DeleteInline::default()),
    )(input)
}

/// Parse a string into a `ReplaceInline` node
fn replace_inline(input: &str) -> IResult<&str, Inline> {
    map(
        tuple((tag(EDIT_START), alt((tag("replace"), tag("rep"))), tag(" "))),
        |_| Inline::ReplaceInline(ReplaceInline::default()),
    )(input)
}

/// Parse a string into a `ModifyInline` node
fn modify_inline(input: &str) -> IResult<&str, Inline> {
    map(
        tuple((tag(EDIT_START), alt((tag("modify"), tag("mod"))), tag(" "))),
        |_| Inline::ModifyInline(ModifyInline::default()),
    )(input)
}

/// Parse a `with:` word indicating the replacement content for a `ReplaceInline` or `ModifyInline` node
fn edit_with(input: &str) -> IResult<&str, Inline> {
    map(tag(EDIT_WITH), |_| Inline::Text(Text::from(EDIT_WITH)))(input)
}

/// Parse double closing square brackets `]]` indicating the end of content
/// for an edit node
fn edit_end(input: &str) -> IResult<&str, Inline> {
    map(tag(EDIT_END), |_| Inline::Text(Text::from(EDIT_END)))(input)
}

/// Accumulate characters into a `Text` node
///
/// Will greedily take as many characters as possible, excluding those that appear at the
/// start of other inline parsers e.g. '$', '['
fn string(input: &str) -> IResult<&str, Inline> {
    const CHARS: &str = "~@#$^&[]{`<>";
    map(
        take_while1(|chr: char| !CHARS.contains(chr)),
        |val: &str| Inline::Text(Text::new(val.into())),
    )(input)
}

/// Take a single character into a `Text` node
///
/// Necessary so that the characters not consumed by `string` are not lost.
fn character(input: &str) -> IResult<&str, Inline> {
    map(take(1usize), |val: &str| {
        Inline::Text(Text::new(val.into()))
    })(input)
}

#[cfg(test)]
mod tests {
    use codec::common::eyre::Result;
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_code_attrs() -> Result<()> {
        code_attrs("``")?;
        code_attrs("``{}")?;
        code_attrs("`a + b`{}")?;
        code_attrs("`a + b`{python}")?;
        code_attrs("`a + b`{python, exec}")?;

        assert!(code_attrs("=`1*1`").is_err());
        assert!(code_attrs("= `2+2`").is_err());

        Ok(())
    }

    #[test]
    fn test_underline() -> Result<()> {
        underline("<u></u>")?;
        underline("<u>underlined</u>")?;

        let inlines = parse_inlines("before <u>underlined</u> after", None);
        assert_eq!(inlines.len(), 3);
        assert!(matches!(inlines[1], Inline::Underline(..)));

        Ok(())
    }

    #[test]
    fn test_instruction_inline() -> Result<()> {
        instruction_inline("[[do something]]")?;
        instruction_inline("[[do something to:")?;

        let inlines = parse_inlines("before [[do something]] after", None);
        assert_eq!(inlines.len(), 3);
        assert!(matches!(inlines[1], Inline::InstructionInline(..)));

        let inlines = parse_inlines("before [[do something to: this]] after", None);
        assert_eq!(inlines.len(), 5);
        assert_eq!(inlines[0], Inline::Text(Text::from("before ")));
        assert!(matches!(inlines[1], Inline::InstructionInline(..)));
        assert_eq!(inlines[2], Inline::Text(Text::from(" this")));
        assert_eq!(inlines[3], Inline::Text(Text::from("]]")));
        assert_eq!(inlines[4], Inline::Text(Text::from(" after")));

        Ok(())
    }
}
