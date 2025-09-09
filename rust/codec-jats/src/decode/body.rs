use std::str::FromStr;

use itertools::Itertools;
use roxmltree::Node;

use stencila_codec::{
    Losses,
    stencila_format::Format,
    stencila_schema::{
        Admonition, Article, AudioObject, AudioObjectOptions, Block, Citation, CitationMode,
        CitationOptions, Claim, ClaimType, CodeBlock, CodeChunk, CodeExpression, CodeInline, Cord,
        CreativeWorkType, Date, DateTime, Duration, ExecutionMode, Figure, Heading, ImageObject,
        ImageObjectOptions, Inline, Link, List, ListItem, ListOrder, MathBlock, MathBlockOptions,
        MathInline, MathInlineOptions, MediaObject, MediaObjectOptions, Note, NoteType, Parameter,
        Section, SectionType, StyledInline, Supplement, Table, TableCell, TableCellOptions,
        TableOptions, TableRow, TableRowType, Text, ThematicBreak, Time, Timestamp, VideoObject,
        VideoObjectOptions,
        shortcuts::{em, mi, p, qb, qi, stg, stk, sub, sup, t, u},
    },
};
use stencila_codec_text_trait::to_text;

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
            "sec" => {
                blocks.append(&mut decode_sec(&child_path, &child, losses, depth + 1));
                continue;
            }
            "statement" => decode_statement(&child_path, &child, losses, depth),
            "supplementary-material" => {
                decode_supplementary_material(&child_path, &child, losses, depth)
            }
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
    if let Some(first) = children.peek()
        && first.tag_name().name() == "caption"
    {
        title = Some(decode_inlines(
            &extend_path(path, "caption"),
            first.children(),
            losses,
        ));
        children.next();
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
/// In addition to [`Paragraph`] nodes, this function may return [`Figure`],
/// [`Table`], [`MathBlock`] or [`Supplement`] nodes, which in JATS can be
/// within a <p> element.
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
        if matches!(
            child_tag,
            "fig" | "table-wrap" | "disp-formula" | "supplementary-material"
        ) {
            if let Some(para) = para(path, children, losses) {
                blocks.push(para);
            }
            children = Vec::new();

            let block = match child_tag {
                "table-wrap" => decode_table_wrap(path, &child, losses, 0),
                "fig" => decode_fig(path, &child, losses, 0),
                "disp-formula" => decode_disp_formula(path, &child, losses, 0),
                "supplementary-material" => decode_supplementary_material(path, &child, losses, 0),
                _ => unreachable!(),
            };
            blocks.push(block);
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
///
/// Some JATS has `<sec>` elements that are merely wrappers with an `id` but
/// no `sec-type` or `<title>` child. These are ignored to avoid unnecessary
/// sections and their content returned instead.
pub fn decode_sec(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Vec<Block> {
    let section_type = node
        .attribute("sec-type")
        .and_then(|value| SectionType::from_text(value).ok())
        .or_else(|| {
            node.children()
                .find(|child| child.tag_name().name() == "title")
                .and_then(|node| node.text())
                .and_then(|value| SectionType::from_text(value).ok())
        });

    record_attrs_lost(path, node, ["sec-type"], losses);

    let content = decode_blocks(path, node.children(), losses, depth);

    if section_type.is_none()
        && !content
            .iter()
            .any(|block| matches!(block, Block::Heading(..)))
    {
        content
    } else {
        vec![Block::Section(Section {
            section_type,
            content,
            ..Default::default()
        })]
    }
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

/// Decode a `<supplementary-material>` element to a Stencila [`Block::Supplement`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/supplementary-material.html
fn decode_supplementary_material(path: &str, node: &Node, losses: &mut Losses, depth: u8) -> Block {
    let id = node.attribute("id").map(String::from);

    record_attrs_lost(path, node, ["id"], losses);

    let mut work_type = None;

    let target = node
        .children()
        .find(|child| child.tag_name().name() == "media")
        .and_then(|node| node.attribute((XLINK, "href")))
        .map(String::from);

    let format = target
        .as_ref()
        .map(|target| Format::from_name(target))
        .unwrap_or_default();

    let target_is_archive = target
        .as_ref()
        .map(|target| target.ends_with(".zip") || target.ends_with(".gz"))
        .unwrap_or_default();

    let target_is_datatable = matches!(
        format,
        Format::Csv | Format::Tsv | Format::Xlsx | Format::Xls
    );

    // Get any label, clean it and try to infer the work type
    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| node.text())
        .map(|label| {
            let rest = label
                .trim()
                .trim_start_matches("Supplement")
                .trim_start_matches("supplement")
                .trim_start_matches("ary")
                .trim_start_matches("al")
                .trim_start_matches("Supporting")
                .trim_start_matches("supporting")
                .trim();

            let rest_lower = rest.to_lowercase();

            if rest_lower.contains("source data") || target_is_archive {
                // Handle labels such as "Figure 2—figure supplement 2—source data 2." (real example)
                // as data not figures: treat as dataset and do not "clean" label
                work_type = Some(CreativeWorkType::Dataset);

                rest.to_string()
            } else if rest_lower.starts_with("table") {
                work_type = Some(if target_is_datatable {
                    CreativeWorkType::Datatable
                } else {
                    CreativeWorkType::Table
                });

                clean_table_label(rest)
            } else if rest_lower.starts_with("fig") {
                work_type = Some(CreativeWorkType::Figure);

                clean_figure_label(rest)
            } else {
                if rest_lower.starts_with("audio") {
                    work_type = Some(CreativeWorkType::AudioObject);
                } else if rest_lower.starts_with("image") {
                    work_type = Some(CreativeWorkType::ImageObject);
                } else if rest_lower.starts_with("video") {
                    work_type = Some(CreativeWorkType::VideoObject);
                } else if rest_lower.starts_with("dataset") {
                    work_type = Some(CreativeWorkType::Dataset);
                }

                rest.to_string()
            }
        });

    let label_automatically = label.is_some().then_some(false);

    let caption = node
        .descendants() // Use descendants because sometimes the caption is nested in <media>
        .find(|child| child.tag_name().name() == "caption")
        .map(|node| decode_blocks(path, node.children(), losses, depth));

    // If work type is still none, attempt to infer from the caption (often
    // there will be no label, only a caption)
    if work_type.is_none()
        && let Some(caption) = &caption
    {
        let caption_lower = to_text(caption).trim().to_lowercase();

        let rest_lower = caption_lower
            .trim_start_matches("supplement")
            .trim_start_matches("ary")
            .trim_start_matches("al")
            .trim_start_matches("supporting")
            .trim();

        if rest_lower.starts_with("figure") {
            work_type = Some(CreativeWorkType::Figure);
        } else if rest_lower.starts_with("table") {
            work_type = Some(CreativeWorkType::Table);
        } else if rest_lower.starts_with("audio") {
            work_type = Some(CreativeWorkType::AudioObject);
        } else if rest_lower.starts_with("image") {
            work_type = Some(CreativeWorkType::ImageObject);
        } else if rest_lower.starts_with("video") {
            work_type = Some(CreativeWorkType::VideoObject);
        } else if rest_lower.starts_with("dataset") {
            work_type = Some(CreativeWorkType::Dataset);
        }
    }

    // If work type is still none, attempt to infer from the format of the target
    if work_type.is_none() {
        if format.is_audio() {
            work_type = Some(CreativeWorkType::AudioObject);
        } else if format.is_image() {
            work_type = Some(CreativeWorkType::ImageObject);
        } else if format.is_video() {
            work_type = Some(CreativeWorkType::VideoObject);
        }
    }

    Block::Supplement(Supplement {
        id,
        work_type,
        label,
        label_automatically,
        caption,
        target,
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
    let id = node.attribute("id").map(String::from);

    record_attrs_lost(path, node, ["id"], losses);

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| node.text())
        .map(clean_figure_label);

    let label_automatically = label.is_some().then_some(false);

    let caption = node
        .children()
        .find(|child| child.tag_name().name() == "caption")
        .map(|node| decode_blocks(path, node.children(), losses, depth))
        .map(clean_caption);

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
        id,
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

        record_attrs_lost(path, node, ["language", "executable", "label-type"], losses);

        let label = node
            .children()
            .find(|child| child.tag_name().name() == "label")
            .and_then(|node| node.text())
            .map(String::from);

        let label_automatically = label.is_some().then_some(false);

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
    let id = node.attribute("id").map(String::from);

    let mut code: Cord = node
        .attribute("code")
        .and_then(|code| code.into())
        .unwrap_or_default()
        .into();

    let mut math_language = node.attribute("language").map(|lang| lang.to_string());

    if code.is_empty()
        && let Some(mathml) = node
            .children()
            .find(|child| child.tag_name().name() == "math")
            .and_then(|node| serialize_node(node).ok())
    {
        code = mathml.into();
        math_language = Some("mathml".into());
    }

    record_attrs_lost(path, node, ["id", "code", "language"], losses);

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| node.text())
        .map(clean_math_block_label);

    let label_automatically = label.is_some().then_some(false);

    let images = if code.is_empty() {
        let images = node
            .descendants() // Use descendants because graphics may be nested in <alternatives>
            .filter(|child| child.tag_name().name() == "graphic")
            .map(|graphic| decode_graphic(&extend_path(path, "graphic"), &graphic, losses))
            .collect_vec();
        (!images.is_empty()).then_some(images)
    } else {
        None
    };

    Block::MathBlock(MathBlock {
        id,
        code,
        math_language,
        label,
        label_automatically,
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
    let id = node.attribute("id").map(String::from);

    record_attrs_lost(path, node, ["id"], losses);

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| node.text())
        .map(clean_table_label);

    let label_automatically = label.is_some().then_some(false);

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
        })
        .map(clean_caption);

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
        .collect_vec();

    let images = if rows.is_empty() {
        let images = node
            .descendants() // Use descendants because graphics may be nested in <alternatives>
            .filter(|child| child.tag_name().name() == "graphic")
            .map(|graphic| decode_graphic(&extend_path(path, "graphic"), &graphic, losses))
            .collect_vec();
        (!images.is_empty()).then_some(images)
    } else {
        None
    };

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
        id,
        label,
        label_automatically,
        caption,
        rows,
        notes,
        options: Box::new(TableOptions {
            images,
            ..Default::default()
        }),
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
    let row_span = node
        .attribute("rowspan")
        .and_then(|alignment| alignment.parse().ok())
        .and_then(|row_span| (row_span != 1).then_some(row_span));

    let column_span = node
        .attribute("colspan")
        .and_then(|alignment| alignment.parse().ok())
        .and_then(|row_span| (row_span != 1).then_some(row_span));

    let vertical_alignment = node
        .attribute("valign")
        .and_then(|alignment| alignment.parse().ok());

    let horizontal_alignment = node
        .attribute("align")
        .and_then(|alignment| alignment.parse().ok());

    record_attrs_lost(
        path,
        node,
        ["rowspan", "colspan", "valign", "align"],
        losses,
    );

    // First try to decode as inlines (usual case) filtering out whitespace only inlines
    let inlines: Vec<Inline> = decode_inlines(path, node.children(), losses)
        .into_iter()
        .filter_map(|inline| (!to_text(&inline).trim().is_empty()).then_some(inline))
        .collect();

    let content = if inlines.is_empty() {
        // Fallback to decoding as blocks, again filtering out whitespace only
        decode_blocks(path, node.children(), losses, depth)
            .into_iter()
            .filter_map(|block| (!to_text(&block).trim().is_empty()).then_some(block))
            .collect()
    } else {
        vec![p(inlines)]
    };

    TableCell {
        content,
        options: Box::new(TableCellOptions {
            row_span,
            column_span,
            vertical_alignment,
            horizontal_alignment,
            ..Default::default()
        }),
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
                    Some("bibr" | "ref") => decode_xref_citation(&child_path, &child, losses),
                    Some(
                        "sec"
                        | "fig"
                        | "table"
                        | "disp-formula"
                        | "supplementary-material"
                        | "media",
                    ) => decode_xref_block(&child_path, &child, losses),
                    _ => {
                        // Record the xref as lost but decode its content
                        record_node_lost(path, &child, losses);
                        inlines.append(&mut decode_inlines(path, child.children(), losses));
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

    record_attrs_lost(path, node, ["code", "language"], losses);

    if code.is_empty()
        && let Some(mathml) = node
            .children()
            .find(|child| child.tag_name().name() == "math")
            .and_then(|node| serialize_node(node).ok())
    {
        code = mathml;
        lang = Some("mathml");
    }

    let images = if code.is_empty() {
        let images = node
            .descendants() // Use descendants because graphics may be nested in <alternatives>
            .filter(|child| child.tag_name().name() == "inline-graphic")
            .map(|graphic| decode_graphic(&extend_path(path, "inline-graphic"), &graphic, losses))
            .collect_vec();
        (!images.is_empty()).then_some(images)
    } else {
        None
    };

    Inline::MathInline(MathInline {
        code: code.into(),
        math_language: lang.map(String::from),
        options: Box::new(MathInlineOptions {
            images,
            ..Default::default()
        }),
        ..Default::default()
    })
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
fn decode_xref_citation(path: &str, node: &Node, losses: &mut Losses) -> Inline {
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

/// Decode a `<xref>` to a [`Inline::Link`] with a target to an internal block
fn decode_xref_block(path: &str, node: &Node, losses: &mut Losses) -> Inline {
    let target = node
        .attribute("rid")
        .map(|id| ["#", id].concat())
        .unwrap_or_default();

    record_attrs_lost(path, node, ["ref-type", "rid"], losses);

    let content = decode_inlines(path, node.children(), losses);

    Inline::Link(Link {
        target,
        content,
        ..Default::default()
    })
}

/// Clean a figure label by removing unnecessary leading and trailing content
fn clean_figure_label(label: &str) -> String {
    const PREFIXES: &[&str] = &["Figure", "figure", "FIGURE", "Fig", "fig", "FIG"];

    let mut cleaned = label;
    for prefix in PREFIXES {
        if let Some(stripped) = cleaned.strip_prefix(prefix) {
            cleaned = stripped;
            break;
        }
    }

    cleaned.trim_matches(['.', ':', ' ']).to_string()
}

/// Clean a table label by removing unnecessary leading and trailing content
fn clean_table_label(label: &str) -> String {
    const PREFIXES: &[&str] = &["Table", "table", "TABLE"];

    let mut cleaned = label;
    for prefix in PREFIXES {
        if let Some(stripped) = cleaned.strip_prefix(prefix) {
            cleaned = stripped;
            break;
        }
    }

    cleaned.trim_matches(['.', ':', ' ']).to_string()
}

/// Clean a match block label by removing unnecessary leading and trailing content
fn clean_math_block_label(label: &str) -> String {
    label
        .trim_start_matches(['(', ' '])
        .trim_end_matches([')', ' '])
        .to_string()
}

/// Clean table & figure captions
///
/// Sometimes a <caption> will have a <title>, which will be decoded to a
/// [Heading]. While a heading is valid content for a caption, that can break
/// downstream assumptions in document structuring and display.
///
/// As such, if the caption starts with a title then we append its content as
/// bolded text to the start of the first paragraph. If there is only a title,
/// then it becomes a paragraph.
fn clean_caption(mut caption: Vec<Block>) -> Vec<Block> {
    if let (Some(Block::Heading(..)), Some(Block::Paragraph(..))) =
        (caption.first(), caption.get(1))
    {
        if let (
            Block::Heading(Heading {
                content: mut heading,
                ..
            }),
            Some(Block::Paragraph(paragraph)),
        ) = (caption.remove(0), caption.get_mut(0))
        {
            if let Some(Inline::Text(text)) = heading.last_mut() {
                if !text.value.ends_with(" ") {
                    text.value.push(' ');
                }
            } else {
                heading.push(t(" "));
            }
            paragraph.content.insert(0, stg(heading));
        }
    } else if caption.len() == 1
        && let Some(Block::Heading(..)) = caption.first()
        && let Block::Heading(Heading { content, .. }) = caption.remove(0)
    {
        caption.push(p(content))
    }

    caption
}
