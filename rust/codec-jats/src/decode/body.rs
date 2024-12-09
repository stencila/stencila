use roxmltree::Node;

use std::str::FromStr;

use codec::{
    schema::{
        shortcuts::{em, mi, p, qb, qi, stg, stk, sub, sup, t, u},
        Admonition, Article, AudioObject, AudioObjectOptions, Block, Claim, ClaimType, CodeBlock,
        CodeChunk, CodeExpression, CodeInline, Cord, Date, DateTime, Duration, ExecutionMode,
        Figure, Heading, ImageObject, ImageObjectOptions, Inline, Link, List, ListItem, ListOrder,
        MathBlock, MediaObject, MediaObjectOptions, Note, NoteType, Parameter, Section,
        StyledInline, ThematicBreak, Time, Timestamp, VideoObject, VideoObjectOptions,
    },
    Losses,
};

use super::utilities::{extend_path, record_attrs_lost, record_node_lost};

const XLINK: &str = "http://www.w3.org/1999/xlink";

/// Decode the `<body>` of an `<article>`
///
/// Iterates over all child elements and either decodes them (by delegating to
/// the corresponding `decode_*` function for the element name), or adds them to
/// losses.
pub(super) fn decode_body(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    article.content = decode_blocks(path, node.children(), losses, 0)
}

/// Decode block content nodes
///
/// Iterates over all child elements and either decodes them, or adds them to
/// losses.
fn decode_blocks<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
    path: &str,
    nodes: I,
    losses: &mut Losses,
    depth: u8,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    for child in nodes {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        let block = match tag {
            "boxed-text" => decode_boxed_text(&child_path, &child, losses, depth),
            "hr" => decode_hr(&child_path, &child, losses),
            "p" => decode_p(&child_path, &child, losses),
            "disp-quote" => decode_disp_quote(&child_path, &child, losses, depth),
            "sec" => decode_sec(&child_path, &child, losses, depth + 1),
            "title" => decode_title(&child_path, &child, losses, depth),
            "list" => decode_list(&child_path, &child, losses, depth),
            "disp-formula" => decode_disp_formula(&child_path, &child, losses, depth),
            "code" => decode_code(&child_path, &child, losses, depth),
            "figure" => decode_figure(&child_path, &child, losses, depth),
            "statement" => decode_statement(&child_path, &child, losses, depth),
            _ => {
                record_node_lost(path, &child, losses);
                continue;
            }
        };
        blocks.push(block);
    }
    blocks
}

/// Decode a `<boxed-text>` to a [`Block::Admonition`]
fn decode_boxed_text(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    record_attrs_lost(path, node, ["content-type", "is-folded"], losses);

    let typ = node
        .attribute("content-type")
        .and_then(|typ| typ.parse().ok())
        .unwrap_or_default();

    let is_folded = node
        .attribute("is-folded")
        .and_then(|is_folded| is_folded.parse().ok());

    let mut title = None;
    let mut children = node.children().peekable();
    if let Some(first) = children.peek() {
        if first.tag_name().name() == "caption" {
            title = Some(decode_inlines(
                &extend_path(path, "caption"),
                first.children(),
                losses,
            ));
            children.next();
        }
    }

    let content = decode_blocks(path, children, losses, depth);

    Block::Admonition(Admonition {
        admonition_type: typ,
        is_folded,
        title,
        content,
        ..Default::default()
    })
}

/// Decode a `<hr>` to a [`Block::ThematicBreak`]
fn decode_hr(path: &str, node: &Node, losses: &mut Losses) -> Block {
    record_attrs_lost(path, node, [], losses);

    Block::ThematicBreak(ThematicBreak::new())
}

/// Decode a `<p>` to a [`Block::Paragraph`]
fn decode_p(path: &str, node: &Node, losses: &mut Losses) -> Block {
    record_attrs_lost(path, node, [], losses);

    p(decode_inlines(path, node.children(), losses))
}

/// Decode a `<disp-quote>` to a [`Block::QuoteBlock`]
fn decode_disp_quote(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    record_attrs_lost(path, node, [], losses);

    qb(decode_blocks(path, node.children(), losses, depth))
}

/// Decode a `<sec>` to a [`Block::Section`]
fn decode_sec(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    record_attrs_lost(path, node, ["content-type"], losses);

    let typ = node
        .attribute("content-type")
        .and_then(|typ| typ.parse().ok());

    Block::Section(Section {
        section_type: typ,
        content: decode_blocks(path, node.children(), losses, depth),
        ..Default::default()
    })
}

/// Decode a `<title>` to a [`Block::Heading`]
fn decode_title(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    record_attrs_lost(path, node, [], losses);

    let level = node
        .attribute("level")
        .and_then(|level| level.parse::<i64>().ok())
        .unwrap_or(depth as i64);

    Block::Heading(Heading::new(
        level,
        decode_inlines(path, node.children(), losses),
    ))
}

/// Decode a `<statement>` to a Stencila [`Block::Claim`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/statement.html
fn decode_statement(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    record_attrs_lost(path, node, ["statement"], losses);
    let claim_type = node
        .attribute("content-type")
        .map(|statement| ClaimType::from_str(statement).unwrap_or_default())
        .unwrap_or_default();
    let label = node.children().find_map(|child| {
        (child.tag_name() == "label".into()).then_some(child.text().unwrap_or_default().to_string())
    });
    Block::Claim(Claim {
        content: decode_blocks(path, node.children(), losses, depth),
        claim_type,
        label,
        ..Default::default()
    })
}

/// Decode a `<figure>` to a Stencila [`Block::Figure`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/figure.html
fn decode_figure(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    record_attrs_lost(path, node, ["figure"], losses);
    Block::Figure(Figure {
        content: decode_blocks(path, node.children(), losses, depth),
        ..Default::default()
    })
}

/// Decode a `<code>` to a Stencila [`Block::CodeBlock`] or Stencila [`Block::CodeChunk`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/code.html
fn decode_code(path: &str, node: &Node, losses: &mut Losses, _depth: u8) -> Block {
    record_attrs_lost(path, node, ["code"], losses);
    let mut programming_language = None;
    let code = node.text().map(Cord::from).unwrap_or_default();
    if let Some(lang) = node.attribute("language").map(|lang| lang.to_string()) {
        programming_language = Some(lang);
    };
    if let Some(execution_mode) = node
        .attribute("executable")
        .map(|mode| ExecutionMode::from_str(mode).ok())
    {
        return Block::CodeChunk(CodeChunk {
            code,
            programming_language,
            execution_mode,
            ..Default::default()
        });
    }
    Block::CodeBlock(CodeBlock {
        code,
        programming_language,
        ..Default::default()
    })
}

/// Decode a `<disp-formula>` to a Stencila [`Block::MathBlock`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/disp-formula.html
fn decode_disp_formula(path: &str, node: &Node, losses: &mut Losses, _depth: u8) -> Block {
    record_attrs_lost(path, node, ["disp-formula"], losses);
    let code = node
        .attribute("code")
        .and_then(|code| code.into())
        .unwrap_or_default();
    let lang = node.attribute("language").map(|lang| lang.to_string());
    Block::MathBlock(MathBlock {
        code: code.into(),
        math_language: lang,
        ..Default::default()
    })
}

/// Decode a `<list>` to a Stencila [`Block::List`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/list.html
fn decode_list(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    record_attrs_lost(path, node, ["list-type"], losses);

    let order = match node.attribute("list-type") {
        // TODO: Encode using valid JATS `list-type`
        // See https://jats.nlm.nih.gov/archiving/tag-library/1.2/attribute/list-type.html
        // Consider adding JATS variants such as `alpha-lower` to `ListOrder`, or
        // adding a new enum for characters to use
        Some("Unordered") | Some("bullet") => ListOrder::Unordered,
        Some("Descending") => ListOrder::Descending,
        _ => ListOrder::Ascending,
    };

    let items = node
        .children()
        .filter_map(|child| {
            let tag = child.tag_name().name();
            if tag == "list-item" {
                Some(decode_list_item(
                    &extend_path(path, tag),
                    &child,
                    losses,
                    depth,
                ))
            } else {
                None
            }
        })
        .collect();

    Block::List(List::new(items, order))
}

/// Decode a `<list-item>` to a Stencila [`ListItem`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/list-item.html
fn decode_list_item(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> ListItem {
    record_attrs_lost(path, node, [], losses);

    ListItem {
        content: decode_blocks(path, node.children(), losses, depth),
        ..Default::default()
    }
}

/// Decode inline content nodes
///
/// Iterates over all child elements and either decodes them, or adds them to
/// losses.
fn decode_inlines<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
    path: &str,
    nodes: I,
    losses: &mut Losses,
) -> Vec<Inline> {
    let mut inlines = Vec::new();
    for child in nodes {
        let inline = if child.is_text() {
            t(child.text().unwrap_or_default())
        } else {
            let tag = child.tag_name().name();
            let child_path = extend_path(path, tag);
            match tag {
                "code" => decode_inline_code(&child_path, &child, losses),
                "date" => decode_date(&child_path, &child, losses),
                "date-time" => decode_date_time(&child_path, &child, losses),
                "duration" => decode_duration(&child_path, &child, losses),
                "ext-link" => decode_link(&child_path, &child, losses),
                "fn" => decode_footnote(&child_path, &child, losses),
                "inline-formula" => decode_math_fragment(&child_path, &child, losses),
                "inline-graphic" | "inline-media" => {
                    decode_inline_media(&child_path, &child, losses)
                }
                "parameter" => decode_parameter(&child_path, &child, losses),
                "styled-content" => decode_styled_content(&child_path, &child, losses),
                "time" => decode_time(&child_path, &child, losses),
                "timestamp" => decode_timestamp(&child_path, &child, losses),
                _ => {
                    record_attrs_lost(&child_path, &child, [], losses);

                    let grandchildren = child.children();
                    match tag {
                        "bold" => stg(decode_inlines(&child_path, grandchildren, losses)),
                        "inline-quote" => qi(decode_inlines(&child_path, grandchildren, losses)),
                        "italic" => em(decode_inlines(&child_path, grandchildren, losses)),
                        "strike" => stk(decode_inlines(&child_path, grandchildren, losses)),
                        "sub" => sub(decode_inlines(&child_path, grandchildren, losses)),
                        "sup" => sup(decode_inlines(&child_path, grandchildren, losses)),
                        "underline" => u(decode_inlines(&child_path, grandchildren, losses)),
                        _ => {
                            record_node_lost(path, &child, losses);
                            continue;
                        }
                    }
                }
            }
        };
        inlines.push(inline);
    }
    inlines
}

/// Decode a `<inline-media>` to a [`Inline::AudioObject`], [`Inline::ImageObject`],
/// or [`Inline::VideoObject`]
///
/// Resolves the destination type based on the `mimetype` attribute of the element.
fn decode_inline_media(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let content_url = node
        .attribute((XLINK, "href"))
        .map(String::from)
        .unwrap_or_default();

    let mime_type = node.attribute("mimetype").map(String::from);
    let mime_subtype = node.attribute("mime-subtype").map(String::from);
    let media_type = match (&mime_type, &mime_subtype) {
        (Some(r#type), Some(subtype)) => Some(format!("{type}/{subtype}")),
        (Some(r#type), None) => Some(r#type.clone()),
        _ => None,
    };

    record_attrs_lost(path, node, ["href", "mimetype", "mime-subtype"], losses);

    let mut caption: Option<Vec<Inline>> = None;
    let mut description = None;
    for child in node.children() {
        let tag = child.tag_name().name();
        match tag {
            "alt-text" => caption = child.text().map(|content| vec![t(content)]),
            "long-desc" => description = child.text().map(String::from),
            _ => record_node_lost(path, &child, losses),
        }
    }

    if node.tag_name().name() == "inline-graphic" {
        return Inline::ImageObject(ImageObject {
            content_url,
            media_type: if media_type.as_deref() == Some("image") {
                None
            } else {
                media_type
            },
            caption,
            options: Box::new(ImageObjectOptions {
                description,
                ..Default::default()
            }),
            ..Default::default()
        });
    }

    match mime_type.as_deref() {
        Some("audio") => Inline::AudioObject(AudioObject {
            content_url,
            media_type: if media_type.as_deref() == Some("audio") {
                None
            } else {
                media_type
            },
            caption,
            options: Box::new(AudioObjectOptions {
                description,
                ..Default::default()
            }),
            ..Default::default()
        }),
        Some("video") => Inline::VideoObject(VideoObject {
            content_url,
            media_type: if media_type.as_deref() == Some("video") {
                None
            } else {
                media_type
            },
            caption,
            options: Box::new(VideoObjectOptions {
                description,
                ..Default::default()
            }),
            ..Default::default()
        }),
        _ => Inline::MediaObject(MediaObject {
            content_url,
            media_type,
            options: Box::new(MediaObjectOptions {
                description,
                ..Default::default()
            }),
            ..Default::default()
        }),
    }
}

/// Decode a `<code>` to a [`Inline::CodeInline`] or [`Inline::CodeExpression`]
fn decode_inline_code(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let executable = node.attribute("executable").map(String::from);
    let programming_language = node.attribute("language").map(String::from);
    let code = node.text().map(Cord::from).unwrap_or_default();

    record_attrs_lost(path, node, ["language"], losses);

    if executable.as_deref() == Some("yes") {
        Inline::CodeExpression(CodeExpression {
            programming_language,
            code,
            ..Default::default()
        })
    } else {
        Inline::CodeInline(CodeInline {
            programming_language,
            code,
            ..Default::default()
        })
    }
}

/// Decode a `<date>` to a [`Inline::Date`]
fn decode_date(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let value = node
        .attribute("iso-8601-date")
        .map(String::from)
        .unwrap_or_default();

    record_attrs_lost(path, node, ["iso-8601-date"], losses);

    Inline::Date(Date {
        value,
        ..Default::default()
    })
}

/// Decode a `<date-time>` to a [`Inline::DateTime`]
fn decode_date_time(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let value = node
        .attribute("iso-8601-date-time")
        .map(String::from)
        .unwrap_or_default();

    record_attrs_lost(path, node, ["iso-8601-date-time"], losses);

    Inline::DateTime(DateTime {
        value,
        ..Default::default()
    })
}

/// Decode a `<duration>` to a [`Inline::Duration`]
fn decode_duration(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let value = node
        .attribute("value")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_default();

    record_attrs_lost(path, node, ["value"], losses);

    Inline::Duration(Duration {
        value,
        ..Default::default()
    })
}

/// Decode a `<ext-link>` to a [`Inline::Link`]
fn decode_link(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let target = node
        .attribute((XLINK, "href"))
        .map(String::from)
        .unwrap_or_default();

    record_attrs_lost(path, node, ["href"], losses);

    let content = decode_inlines(path, node.children(), losses);

    Inline::Link(Link {
        target,
        content,
        ..Default::default()
    })
}

/// Decode a `<fn>` to a [`Inline::Footnote`]
fn decode_footnote(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let fn_type = node
        .attribute("fn-type")
        .map(String::from)
        .unwrap_or_default();

    let custom_type = node
        .attribute("custom-type")
        .map(String::from)
        .unwrap_or_default();

    let note_type = if fn_type == "custom" {
        match custom_type.to_lowercase().as_str() {
            "endnote" => NoteType::Endnote,
            "sidenote" => NoteType::Sidenote,
            _ => NoteType::Footnote,
        }
    } else {
        NoteType::Footnote
    };

    record_attrs_lost(path, node, ["fn-type", "custom-type"], losses);

    let content = decode_blocks(path, node.children(), losses, 0);

    Inline::Note(Note {
        note_type,
        content,
        ..Default::default()
    })
}

/// Decode a `<inline-formula>` to a [`Inline::MathInline`]
fn decode_math_fragment(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let code = node.attribute("code").map(Cord::from).unwrap_or_default();
    let lang = node.attribute("language");

    record_attrs_lost(path, node, ["code", "language"], losses);

    mi(code, lang)
}

/// Decode a `<parameter>` to a [`Inline::Parameter`]
fn decode_parameter(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let name = node.attribute("name").map(String::from).unwrap_or_default();

    record_attrs_lost(path, node, ["name"], losses);

    Inline::Parameter(Parameter {
        name,
        ..Default::default()
    })
}

/// Decode a `<styled-content>` to a [`Inline::StyledInline`]
fn decode_styled_content(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let code = node.attribute("style").map(Cord::from).unwrap_or_default();

    let style_language = node.attribute("style-detail").map(String::from);

    record_attrs_lost(path, node, ["style", "style-detail"], losses);

    let content = decode_inlines(path, node.children(), losses);

    Inline::StyledInline(StyledInline {
        code,
        style_language,
        content,
        ..Default::default()
    })
}

/// Decode a `<time>` to a [`Inline::Time`]
fn decode_time(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let value = node
        .attribute("iso-8601-time")
        .map(String::from)
        .unwrap_or_default();

    record_attrs_lost(path, node, ["iso-8601-time"], losses);

    Inline::Time(Time {
        value,
        ..Default::default()
    })
}

/// Decode a `<timestamp>` to a [`Inline::Timestamp`]
fn decode_timestamp(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let value = node
        .attribute("value")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_default();

    record_attrs_lost(path, node, ["value"], losses);

    Inline::Timestamp(Timestamp {
        value,
        ..Default::default()
    })
}
