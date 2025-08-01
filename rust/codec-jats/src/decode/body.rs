use roxmltree::Node;

use std::str::FromStr;

use codec::{
    Losses,
    common::itertools::Itertools,
    schema::{
        Admonition, Article, AudioObject, AudioObjectOptions, Block, Citation, CitationMode,
        CitationOptions, Claim, ClaimType, CodeBlock, CodeChunk, CodeExpression, CodeInline, Cord,
        Date, DateTime, Duration, ExecutionMode, Figure, Heading, ImageObject, ImageObjectOptions,
        Inline, Link, List, ListItem, ListOrder, MathBlock, MathBlockOptions, MediaObject,
        MediaObjectOptions, Note, NoteType, Parameter, Section, SectionType, StyledInline, Table,
        TableCell, TableRow, TableRowType, Text, ThematicBreak, Time, Timestamp, VideoObject,
        VideoObjectOptions,
        shortcuts::{em, mi, p, qb, qi, stg, stk, sub, sup, t, u},
    },
};

use crate::encode::serialize_node;

use super::{
    inlines::normalize_inlines,
    utilities::{extend_path, record_attrs_lost, record_node_lost},
};

const XLINK: &str = "http://www.w3.org/1999/xlink";

/// Decode the `<body>` of an `<article>`
///
/// Iterates over all child elements and either decodes them (by delegating to
/// the corresponding `decode_*` function for the element name), or adds them to
/// losses.
pub(super) fn decode_body(path: &str, node: &Node, article: &mut Article, losses: &mut Losses) {
    let mut content = decode_blocks(path, node.children(), losses, 0);
    article.content.append(&mut content)
}

/// Decode block content nodes
///
/// Iterates over all child elements and either decodes them, or adds them to
/// losses.
pub(super) fn decode_blocks<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
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
            "code" => decode_code(&child_path, &child, losses, depth),
            "disp-formula" => decode_disp_formula(&child_path, &child, losses, depth),
            "disp-quote" => decode_disp_quote(&child_path, &child, losses, depth),
            "fig" => decode_fig(&child_path, &child, losses, depth),
            "fig-group" => {
                blocks.append(&mut decode_fig_group(&child_path, &child, losses, depth));
                continue;
            }
            "fn" => {
                blocks.append(&mut decode_fn(&child_path, &child, losses, depth));
                continue;
            }
            "graphic" => {
                blocks.push(Block::ImageObject(decode_graphic(
                    &child_path,
                    &child,
                    losses,
                )));
                continue;
            }
            "hr" => decode_hr(&child_path, &child, losses),
            "list" => decode_list(&child_path, &child, losses, depth),
            "p" => {
                blocks.append(&mut decode_p(&child_path, &child, losses));
                continue;
            }
            "sec" => decode_sec(&child_path, &child, losses, depth + 1),
            "statement" => decode_statement(&child_path, &child, losses, depth),
            "title" => decode_title(&child_path, &child, losses, depth),
            "table-wrap" => decode_table_wrap(&child_path, &child, losses, depth),
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

    record_attrs_lost(path, node, ["content-type", "is-folded"], losses);

    Block::Admonition(Admonition {
        admonition_type: typ,
        is_folded,
        title,
        content: decode_blocks(path, children, losses, depth),
        ..Default::default()
    })
}

/// Decode a `<hr>` to a [`Block::ThematicBreak`]
fn decode_hr(path: &str, node: &Node, losses: &mut Losses) -> Block {
    record_attrs_lost(path, node, [], losses);

    Block::ThematicBreak(ThematicBreak::new())
}

/// Decode a <p> element to a vector of blocks
///
/// In addition to [`Paragraph`]nodes, this function may return [`MathBlock`] nodes,
/// which in JATS can be within a <p> element.
fn decode_p(path: &str, node: &Node, losses: &mut Losses) -> Vec<Block> {
    record_attrs_lost(path, node, [], losses);

    /// Create a paragraph using collected nodes
    fn para(path: &str, children: Vec<Node>, losses: &mut Losses) -> Option<Block> {
        let inlines = decode_inlines(path, children.into_iter(), losses);

        if !(inlines.is_empty()
            || inlines.iter().all(|inline| match inline {
                Inline::Text(Text { value, .. }) => value.trim().is_empty(),
                _ => false,
            }))
        {
            Some(p(inlines))
        } else {
            None
        }
    }

    let mut blocks = Vec::new();
    let mut children = Vec::new();
    for child in node.children() {
        let child_tag = child.tag_name().name();
        if child_tag == "disp-formula" {
            if let Some(para) = para(path, children, losses) {
                blocks.push(para);
            }
            children = Vec::new();

            let math_block = decode_disp_formula(path, &child, losses, 0);
            blocks.push(math_block);
        } else {
            children.push(child);
        }
    }

    if let Some(para) = para(path, children, losses) {
        blocks.push(para);
    }

    blocks
}

/// Decode a `<disp-quote>` to a [`Block::QuoteBlock`]
fn decode_disp_quote(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    record_attrs_lost(path, node, [], losses);

    qb(decode_blocks(path, node.children(), losses, depth))
}

/// Decode a `<sec>` to a [`Block::Section`]
fn decode_sec(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    fn parse_section_type(section_type: &str) -> Option<SectionType> {
        section_type
            .parse()
            .ok()
            .or_else(|| match section_type.to_lowercase().as_str() {
                "intro" => Some(SectionType::Introduction),
                section_type => {
                    if section_type.contains("methods") {
                        Some(SectionType::Methods)
                    } else {
                        None
                    }
                }
            })
    }

    let section_type = node
        .attribute("sec-type")
        .and_then(parse_section_type)
        .or_else(|| {
            node.children()
                .find(|child| child.tag_name().name() == "title")
                .and_then(|node| node.text())
                .and_then(parse_section_type)
        });

    record_attrs_lost(path, node, ["sec-type"], losses);

    Block::Section(Section {
        section_type,
        content: decode_blocks(path, node.children(), losses, depth),
        ..Default::default()
    })
}

/// Decode a `<title>` to a [`Block::Heading`]
fn decode_title(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    let level = node
        .attribute("level")
        .and_then(|level| level.parse::<i64>().ok())
        .unwrap_or(depth as i64);

    record_attrs_lost(path, node, ["level"], losses);

    Block::Heading(Heading::new(
        level,
        decode_inlines(path, node.children(), losses),
    ))
}

/// Decode a `<statement>` to a Stencila [`Block::Claim`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/statement.html
fn decode_statement(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    let claim_type = node
        .attribute("content-type")
        .map(|statement| ClaimType::from_str(statement).unwrap_or_default())
        .unwrap_or_default();

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| node.text())
        .map(String::from);

    let content = decode_blocks(
        path,
        node.children()
            .filter(|child| child.tag_name().name() != "label"),
        losses,
        depth,
    );

    record_attrs_lost(path, node, ["content-type"], losses);

    Block::Claim(Claim {
        claim_type,
        label,
        content,
        ..Default::default()
    })
}

/// Decode a `<fig-group>` element to a vector of Stencila [`Block::Figure`]s
fn decode_fig_group(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Vec<Block> {
    record_attrs_lost(path, node, [], losses);

    decode_blocks(path, node.children(), losses, depth)
}

/// Decode a `<fig>` element to a Stencila [`Block::Figure`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/fig.html
fn decode_fig(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    let label_automatically = node
        .attribute("label-automatically")
        .and_then(|string| string.parse().ok());

    record_attrs_lost(path, node, ["label-automatically"], losses);

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| node.text())
        .map(|label| {
            label
                .trim_start_matches("Figure ")
                .trim_end_matches(['.', ':', ' '])
        })
        .map(String::from);

    let caption = node
        .children()
        .find(|child| child.tag_name().name() == "caption")
        .map(|node| decode_blocks(path, node.children(), losses, depth));

    // Decode remaining blocks (not <label> or <caption>)
    let content = decode_blocks(
        path,
        node.children().filter(|child| {
            let tag_name = child.tag_name().name();
            tag_name != "label" && tag_name != "caption"
        }),
        losses,
        depth,
    );

    Block::Figure(Figure {
        content,
        caption,
        label_automatically,
        label,
        ..Default::default()
    })
}

/// Decode a `<fn>` element to a vector of Stencila [`Block`]s
fn decode_fn(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Vec<Block> {
    // TODO: attach the decoded blocks to a Stencila `Note`
    record_attrs_lost(path, node, [], losses);

    decode_blocks(path, node.children(), losses, depth)
}

/// Decode a `<graphic>` element to an [`ImageObject`]
fn decode_graphic(path: &str, node: &Node, losses: &mut Losses) -> ImageObject {
    let url = node
        .attribute((XLINK, "href"))
        .map(String::from)
        .unwrap_or_default();

    record_attrs_lost(path, node, ["href"], losses);

    ImageObject {
        content_url: url,
        ..Default::default()
    }
}

/// Decode a `<code>` to a Stencila [`Block::CodeBlock`] or Stencila [`Block::CodeChunk`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/code.html
fn decode_code(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    let code = node.text().map(Cord::from).unwrap_or_default();

    let programming_language = node.attribute("language").map(String::from);

    if let Some(execution_mode) = node
        .attribute("executable")
        .map(|mode| ExecutionMode::from_str(mode).ok())
    {
        let label_type = node
            .attribute("label-type")
            .and_then(|string| string.parse().ok());

        let label_automatically = node
            .attribute("label-automatically")
            .and_then(|string| string.parse().ok());

        record_attrs_lost(
            path,
            node,
            [
                "language",
                "executable",
                "label-type",
                "label-automatically",
            ],
            losses,
        );

        let label = node
            .children()
            .find(|child| child.tag_name().name() == "label")
            .and_then(|node| node.text())
            .map(String::from);

        let caption = node
            .children()
            .find(|child| child.tag_name().name() == "caption")
            .map(|node| decode_blocks(path, node.children(), losses, depth));

        return Block::CodeChunk(CodeChunk {
            code,
            programming_language,
            execution_mode,
            caption,
            label,
            label_type,
            label_automatically,
            ..Default::default()
        });
    }

    record_attrs_lost(path, node, ["language"], losses);

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
    let mut code = node
        .attribute("code")
        .and_then(|code| code.into())
        .unwrap_or_default()
        .to_string();

    let mut math_language = node.attribute("language").map(|lang| lang.to_string());

    if code.is_empty() {
        if let Some(mathml) = node
            .children()
            .find(|child| child.tag_name().name() == "math")
            .and_then(|node| serialize_node(node).ok())
        {
            code = mathml;
            math_language = Some("mathml".into());
        }
    }

    let images = node
        .children()
        .filter(|child| child.tag_name().name() == "graphic")
        .map(|graphic| decode_graphic(&extend_path(path, "graphic"), &graphic, losses))
        .collect_vec();
    let images = (!images.is_empty()).then_some(images);

    record_attrs_lost(path, node, ["code", "language"], losses);

    Block::MathBlock(MathBlock {
        code: code.into(),
        math_language,
        options: Box::new(MathBlockOptions {
            images,
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Decode a `<list>` to a Stencila [`Block::List`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/list.html
fn decode_list(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    let order = match node.attribute("list-type") {
        // TODO: Encode using valid JATS `list-type`
        // See https://jats.nlm.nih.gov/archiving/tag-library/1.2/attribute/list-type.html
        // Consider adding JATS variants such as `alpha-lower` to `ListOrder`, or
        // adding a new enum for characters to use
        Some("Unordered") | Some("bullet") => ListOrder::Unordered,
        Some("Descending") => ListOrder::Descending,
        _ => ListOrder::Ascending,
    };

    record_attrs_lost(path, node, ["list-type"], losses);

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
    let is_checked = node
        .attribute("is-checked")
        .and_then(|value| bool::from_str(value).ok());

    record_attrs_lost(path, node, ["is-checked"], losses);

    ListItem {
        is_checked,
        content: decode_blocks(path, node.children(), losses, depth),
        ..Default::default()
    }
}

/// Decode a `<table-wrap>` to a Stencila [`Block::Table`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/table-wrap.html
fn decode_table_wrap(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    let label_automatically = node
        .attribute("label-automatically")
        .and_then(|string| string.parse().ok());

    record_attrs_lost(path, node, ["label-automatically"], losses);

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| node.text())
        .map(|label| {
            label
                .trim_start_matches("Table ")
                .trim_end_matches(['.', ':', ' '])
        })
        .map(String::from);

    let caption = node
        .children()
        .find(|child| child.tag_name().name() == "caption")
        .map(|node| {
            decode_blocks(
                &extend_path(path, "caption"),
                node.children(),
                losses,
                depth,
            )
        });

    let rows = node
        .children()
        .filter(|child| child.tag_name().name() == "table")
        .flat_map(|child| {
            let path = &extend_path(path, "table");
            child
                .children()
                .flat_map(|grandchild| {
                    let tag = grandchild.tag_name().name();
                    let grandchild_path = extend_path(path, tag);
                    let row_type = match tag {
                        "thead" => Some(TableRowType::HeaderRow),
                        "tfoot" => Some(TableRowType::FooterRow),
                        _ => None,
                    };
                    if tag == "thead" || tag == "tbody" || tag == "tfoot" {
                        decode_table_section(&grandchild_path, &grandchild, losses, depth, row_type)
                    } else if tag == "tr" {
                        vec![decode_table_row(path, &grandchild, losses, depth, None)]
                    } else {
                        Vec::new()
                    }
                })
                .collect::<Vec<TableRow>>()
        })
        .collect();

    let notes = node
        .children()
        .find(|child| child.tag_name().name() == "table-wrap-foot")
        .map(|node| {
            decode_blocks(
                &extend_path(path, "table-wrap-foot"),
                node.children(),
                losses,
                depth,
            )
        });

    Block::Table(Table {
        label,
        label_automatically,
        caption,
        rows,
        notes,
        ..Default::default()
    })
}

/// Decode a `<thead>`,`<tbody>`, or `<tfoot>` to a Stencila [`Vec<TableRow>`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/thead.html,
/// https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/tbody.html,
/// and https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/tfoot.html
fn decode_table_section(
    path: &str,
    node: &Node,
    losses: &mut Losses,
    depth: u8,
    row_type: Option<TableRowType>,
) -> Vec<TableRow> {
    node.children()
        .filter(|child| child.tag_name().name() == "tr")
        .map(|child| decode_table_row(path, &child, losses, depth, row_type))
        .collect()
}

/// Decode a `<tr>` to a Stencila [`TableRow`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/tr.html
fn decode_table_row(
    path: &str,
    node: &Node,
    losses: &mut Losses,
    depth: u8,
    row_type: Option<TableRowType>,
) -> TableRow {
    record_attrs_lost(path, node, [], losses);

    let mut cells = Vec::new();
    let path = &extend_path(path, "tr");

    for child in node.children() {
        if child.tag_name().name() == "td" {
            cells.push(decode_table_cell(
                &extend_path(path, "td"),
                &child,
                losses,
                depth,
            ));
        } else if child.tag_name().name() == "th" {
            cells.push(decode_table_cell(
                &extend_path(path, "th"),
                &child,
                losses,
                depth,
            ));
        }
    }

    TableRow {
        cells,
        row_type,
        ..Default::default()
    }
}

/// Decode a `<td>` or `<th>` to a Stencila [`TableCell`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/td.html
/// and https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/th.html
fn decode_table_cell(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> TableCell {
    let vertical_alignment = node
        .attribute("valign")
        .and_then(|alignment| alignment.parse().ok());
    let horizontal_alignment = node
        .attribute("align")
        .and_then(|alignment| alignment.parse().ok());

    record_attrs_lost(path, node, ["valign", "align"], losses);

    let mut content = vec![p(decode_inlines(path, node.children(), losses))];
    if decode_inlines(path, node.children(), losses).is_empty() {
        content = decode_blocks(path, node.children(), losses, depth);
    }

    TableCell {
        content,
        vertical_alignment,
        horizontal_alignment,
        ..Default::default()
    }
}

/// Decode inline content nodes
///
/// Iterates over all child elements and either decodes them, or adds them to
/// losses.
pub fn decode_inlines<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
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
                "inline-formula" => decode_inline_formula(&child_path, &child, losses),
                "inline-graphic" | "inline-media" => {
                    decode_inline_media(&child_path, &child, losses)
                }
                "math" => decode_inline_math(&child),
                "parameter" => decode_parameter(&child_path, &child, losses),
                "styled-content" => decode_styled_content(&child_path, &child, losses),
                "time" => decode_time(&child_path, &child, losses),
                "timestamp" => decode_timestamp(&child_path, &child, losses),
                "xref" => match child.attribute("ref-type") {
                    Some("bibr" | "ref") => decode_xref_bibr(&child_path, &child, losses),
                    _ => {
                        record_node_lost(path, &child, losses);
                        continue;
                    }
                },
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

    normalize_inlines(inlines)
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

    record_attrs_lost(path, node, ["language", "executable"], losses);

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
fn decode_inline_formula(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let mut code = node.attribute("code").unwrap_or_default().to_string();
    let mut lang = node.attribute("language");

    if code.is_empty() {
        if let Some(mathml) = node
            .children()
            .find(|child| child.tag_name().name() == "math")
            .and_then(|node| serialize_node(node).ok())
        {
            code = mathml;
            lang = Some("mathml");
        }
    }

    record_attrs_lost(path, node, ["code", "language"], losses);

    mi(code, lang)
}

/// Decode a inline `<mml::math>` element to a [`Inline::MathInline`]
fn decode_inline_math(node: &Node) -> Inline {
    let code = serialize_node(*node).unwrap_or_default();
    let lang = Some("mathml");

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

/// Decode a `<xref>` with `ref-type` of "bibr" or "ref" to a [`Inline::Citation`]
fn decode_xref_bibr(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let target = node.attribute("rid").map(String::from).unwrap_or_default();

    record_attrs_lost(path, node, ["ref-type", "rid"], losses);

    let content = decode_inlines(path, node.children(), losses);
    let content = (!content.is_empty()).then_some(content);

    Inline::Citation(Citation {
        target,
        citation_mode: Some(CitationMode::Parenthetical),
        options: Box::new(CitationOptions {
            content,
            ..Default::default()
        }),
        ..Default::default()
    })
}
