use super::DecodeContext;

use std::{iter::once, str::FromStr};

use itertools::Itertools;
use roxmltree::Node;

use stencila_codec::{
    stencila_format::Format,
    stencila_schema::{
        Admonition, AudioObject, AudioObjectOptions, Block, Citation, CitationGroup,
        CitationOptions, Claim, ClaimType, CodeBlock, CodeChunk, CodeExpression, CodeInline, Cord,
        CreativeWorkType, Date, DateTime, Duration, Figure, Heading, HorizontalAlignment,
        ImageObject, ImageObjectOptions, Inline, Link, List, ListItem, ListOrder, MathBlock,
        MathBlockOptions, MathInline, MathInlineOptions, MediaObject, MediaObjectOptions, Note,
        NoteType, Parameter, Section, SectionType, StyledInline, Supplement, Table, TableCell,
        TableCellOptions, TableCellType, TableOptions, TableRow, TableRowType, Text, ThematicBreak,
        Time, Timestamp, VerticalAlignment, VideoObject, VideoObjectOptions,
        shortcuts::{em, mi, p, qb, qi, stg, stk, sub, sup, t, u},
    },
};
use stencila_codec_jats_trait::{JatsRefType, encode::escape, has_uri_scheme};
use stencila_codec_text_trait::to_text;

use crate::encode::serialize_node;

use super::{
    back::decode_ref_list,
    utilities::{XLINK, extend_path, non_empty_text_deep, record_attrs_lost, record_node_lost},
};

/// Whether a JATS element is decoded by [`decode_blocks`] into block content
///
/// Deliberately excludes elements such as `<fn>` and `<title>` which are
/// decoded as inline content in an inline context. Used to choose between
/// inline and block decoding without a trial decode.
fn is_block_element(tag: &str) -> bool {
    matches!(
        tag,
        "boxed-text"
            | "code"
            | "disp-formula"
            | "disp-quote"
            | "fig"
            | "fig-group"
            | "fn-group"
            | "hr"
            | "list"
            | "p"
            | "sec"
            | "statement"
            | "supplementary-material"
            | "table-wrap"
    )
}

/// Whether a JATS element within a `<p>` should end the paragraph and be
/// decoded as a block in its own right
///
/// Excludes `<code>`, which JATS allows as a block within a paragraph but which
/// Stencila also emits for inline code, so treating it as a block would break
/// the round trip of inline code.
fn splits_paragraph(tag: &str) -> bool {
    is_block_element(tag) && !matches!(tag, "code" | "p")
}

/// Whether a JATS element within a `<td>` or `<th>` should be decoded as a
/// block in its own right
///
/// Excludes `<code>` for the same reason as [`splits_paragraph`]. Unlike within
/// a paragraph, a `<p>` within a cell is a block.
fn is_cell_block(tag: &str) -> bool {
    is_block_element(tag) && tag != "code"
}

/// Determine the [`SectionType`] of a `<sec>`, `<notes>` or `<fn-group>`, and
/// record the attributes of it that are lost
///
/// The type is taken from `attribute` (e.g. `sec-type`) when that has a value
/// that maps to a type, and otherwise from the `<title>`. The attribute is
/// only treated as decoded when it maps to a type, so that a value with no
/// equivalent (e.g. `additional-information`) is reported as lost. Other
/// decoded attributes are passed in `handled`.
pub(super) fn section_type<const N: usize>(
    path: &str,
    node: &Node,
    attribute: &str,
    handled: [&str; N],
    losses: &mut DecodeContext,
) -> Option<SectionType> {
    let from_attribute = node.attribute(attribute).and_then(jats_section_type);

    let mut not_lost = handled.to_vec();
    if from_attribute.is_some() {
        not_lost.push(attribute);
    }
    record_attrs_lost(path, node, not_lost, losses);

    from_attribute.or_else(|| {
        node.children()
            .find(|child| child.has_tag_name("title"))
            .and_then(|title| non_empty_text_deep(&title))
            .and_then(|title| SectionType::from_text(&title).ok())
    })
}

/// Map a JATS section type to a [`SectionType`]
///
/// JATS writes a section that is of more than one type as the types separated
/// by `|`, as in the recommended `materials|methods`. That is read as the
/// combination (e.g. "materials and methods") if that has a type, and otherwise
/// as the first of the types that has one.
fn jats_section_type(value: &str) -> Option<SectionType> {
    let value = value.trim();
    SectionType::from_text(&value.replace('|', " and "))
        .ok()
        .or_else(|| {
            value
                .split('|')
                .find_map(|part| SectionType::from_text(part).ok())
        })
}

/// Decode block content nodes
///
/// Iterates over all child elements and either decodes them, or adds them to
/// losses.
pub(super) fn decode_blocks<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
    path: &str,
    nodes: I,
    losses: &mut DecodeContext,
    depth: u8,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    for child in nodes {
        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        let block = match tag {
            "boxed-text" => decode_boxed_text(&child_path, &child, losses, depth),
            "code" => decode_code(&child_path, &child, losses, depth),
            "disp-formula" => decode_disp_formula(&child_path, &child, losses),
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
            "fn-group" => {
                blocks.append(&mut decode_fn_group(&child_path, &child, losses, depth));
                continue;
            }
            "alternatives" => {
                record_attrs_lost(&child_path, &child, [], losses);
                match select_alternative(&child, &["graphic", "media", "table-wrap", "code"]) {
                    Some(alternative) => blocks.append(&mut decode_blocks(
                        &child_path,
                        once(alternative),
                        losses,
                        depth,
                    )),
                    None => record_node_lost(path, &child, losses),
                }
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
            "media" | "inline-media" => {
                blocks.push(p(vec![decode_inline_media(&child_path, &child, losses)]));
                continue;
            }
            "hr" => decode_hr(&child_path, &child, losses),
            "list" => decode_list(&child_path, &child, losses, depth),
            "p" => {
                blocks.append(&mut decode_p(&child_path, &child, losses, depth));
                continue;
            }
            "ref-list" => {
                decode_ref_list(&child_path, &child, losses);
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
fn decode_boxed_text(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Block {
    let typ = node
        .attribute("content-type")
        .and_then(|typ| typ.parse().ok())
        .unwrap_or_default();

    let is_folded = node
        .attribute("is-folded")
        .and_then(|is_folded| is_folded.parse().ok());

    let caption = node.children().find(|child| child.has_tag_name("caption"));
    let title = caption
        .map(|caption| decode_inlines(&extend_path(path, "caption"), caption.children(), losses));

    record_attrs_lost(path, node, ["id", "content-type", "is-folded"], losses);

    Block::Admonition(Admonition {
        id: node.attribute("id").map(String::from),
        admonition_type: typ,
        is_folded,
        title,
        content: decode_blocks(
            path,
            node.children().filter(|child| Some(*child) != caption),
            losses,
            depth,
        ),
        ..Default::default()
    })
}

/// Decode a `<hr>` to a [`Block::ThematicBreak`]
fn decode_hr(path: &str, node: &Node, losses: &mut DecodeContext) -> Block {
    record_attrs_lost(path, node, [], losses);

    Block::ThematicBreak(ThematicBreak::new())
}

/// Decode a <p> element to a vector of blocks
///
/// JATS allows many block elements (lists, boxed text, quotes, figures, tables,
/// formulas and supplementary material amongst them) to appear inside a `<p>`.
/// Any such child ends the paragraph being accumulated, is decoded as a block in
/// its own right, and a new paragraph is started for the inline content that
/// follows it. Note that `<fn>` is deliberately not treated as a block child
/// here: within a paragraph it is a footnote anchored at that point and is
/// decoded as an inline [`Note`].
///
/// The `id` of the `<p>` goes to the first paragraph decoded from it, so that
/// it is only kept if the `<p>` starts with inline content.
fn decode_p(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Vec<Block> {
    record_attrs_lost(path, node, ["id"], losses);

    /// Create a paragraph using collected nodes
    fn para(path: &str, children: Vec<Node>, losses: &mut DecodeContext) -> Option<Block> {
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
        if child.is_element() && splits_paragraph(child_tag) {
            if let Some(para) = para(path, children, losses) {
                blocks.push(para);
            }
            children = Vec::new();

            blocks.append(&mut decode_blocks(path, once(child), losses, depth));
        } else {
            children.push(child);
        }
    }

    if let Some(para) = para(path, children, losses) {
        blocks.push(para);
    }

    if let Some(id) = node.attribute("id") {
        match blocks.first_mut() {
            Some(Block::Paragraph(paragraph)) => paragraph.id = Some(id.to_string()),
            _ => losses.add(format!("{path}/@id")),
        }
    }

    blocks
}

/// Decode a `<disp-quote>` to a [`Block::QuoteBlock`]
fn decode_disp_quote(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Block {
    record_attrs_lost(path, node, [], losses);

    qb(decode_blocks(path, node.children(), losses, depth))
}

/// Decode a `<sec>` to a [`Block::Section`]
///
/// Some JATS has `<sec>` elements that are merely wrappers with no `id`,
/// `sec-type` or `<title>` child. These are ignored to avoid unnecessary
/// sections and their content returned instead. A wrapper that has an `id` is
/// kept because cross references elsewhere in the document may address it.
pub fn decode_sec(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Vec<Block> {
    let id = node.attribute("id").map(String::from);

    let section_type = section_type(path, node, "sec-type", ["id"], losses);

    let content = decode_blocks(path, node.children(), losses, depth);

    if id.is_none()
        && section_type.is_none()
        && !content
            .iter()
            .any(|block| matches!(block, Block::Heading(..)))
    {
        content
    } else {
        vec![Block::Section(Section {
            id,
            section_type,
            content,
            ..Default::default()
        })]
    }
}

/// Decode a `<title>` to a [`Block::Heading`]
fn decode_title(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Block {
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
fn decode_statement(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Block {
    let claim_type = node
        .attribute("content-type")
        .and_then(|statement| ClaimType::from_str(statement).ok());

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| non_empty_text_deep(&node));

    let content = decode_blocks(
        path,
        node.children()
            .filter(|child| child.tag_name().name() != "label"),
        losses,
        depth,
    );

    record_attrs_lost(path, node, ["id", "content-type"], losses);

    Block::Claim(Claim {
        id: node.attribute("id").map(String::from),
        claim_type,
        label,
        content,
        ..Default::default()
    })
}

/// Decode a `<supplementary-material>` element to a Stencila [`Block::Supplement`]
///
/// The type of the work is taken from the `content-type`, which Stencila emits,
/// and otherwise inferred from the label, the caption, the stated media type
/// and the target, in that order.
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/supplementary-material.html
fn decode_supplementary_material(
    path: &str,
    node: &Node,
    losses: &mut DecodeContext,
    depth: u8,
) -> Block {
    let id = node.attribute("id").map(String::from);

    let mut work_type = node
        .attribute("content-type")
        .and_then(|value| CreativeWorkType::from_str(value).ok());

    let mut not_lost = vec!["id", "href", "mimetype", "mime-subtype"];
    if work_type.is_some() {
        not_lost.push("content-type");
    }
    record_attrs_lost(path, node, not_lost, losses);

    // The resource may be stated as a `<media>` or, for an image, as a
    // `<graphic>`; either way it is what the supplement points at
    let resource = node
        .children()
        .find(|child| child.has_tag_name("media") || child.has_tag_name("graphic"));
    if let Some(resource) = &resource {
        record_attrs_lost(
            &extend_path(path, resource.tag_name().name()),
            resource,
            ["href", "mimetype", "mime-subtype"],
            losses,
        );
    }

    let target_node = resource
        .as_ref()
        .or_else(|| node.attribute((XLINK, "href")).map(|_| node));
    let resolved = target_node.map(|node| losses.media(node));
    let target = resolved
        .as_ref()
        .map(|(url, _)| url.clone())
        .filter(|url| !url.is_empty());

    // The target may be a URL, so any query or fragment is ignored
    let target_path = target
        .as_deref()
        .and_then(|target| target.split(['?', '#']).next())
        .unwrap_or_default();
    let format = if target_path.is_empty() {
        Format::default()
    } else {
        Format::from_url(target_path)
    };
    let target_is_archive = target_path.ends_with(".zip") || target_path.ends_with(".gz");
    let target_is_datatable = matches!(
        format,
        Format::Csv | Format::Tsv | Format::Xlsx | Format::Xls
    );

    // Get any label, clean it and try to infer the work type
    let label_node = node.children().find(|child| child.has_tag_name("label"));
    let label = label_node
        .and_then(|node| non_empty_text_deep(&node))
        .map(|label| {
            let rest = strip_supplement_prefix(&label);
            let rest_lower = rest.to_lowercase();

            let (inferred, label) = if rest_lower.contains("source data") || target_is_archive {
                // Handle labels such as "Figure 2—figure supplement 2—source data 2." (real example)
                // as data not figures: treat as dataset and do not "clean" label
                (Some(CreativeWorkType::Dataset), rest.to_string())
            } else if rest_lower.starts_with("table") {
                (
                    Some(if target_is_datatable {
                        CreativeWorkType::Datatable
                    } else {
                        CreativeWorkType::Table
                    }),
                    clean_label(rest, TABLE_PREFIXES),
                )
            } else if rest_lower.starts_with("fig") {
                (
                    Some(CreativeWorkType::Figure),
                    clean_label(rest, FIGURE_PREFIXES),
                )
            } else {
                (work_type_of(&rest_lower), rest.to_string())
            };
            if work_type.is_none() {
                work_type = inferred;
            }
            label
        });

    let label_automatically = label.is_some().then_some(false);

    // Use descendants because sometimes the caption is nested in <media>
    let caption_node = node
        .descendants()
        .find(|child| child.tag_name().name() == "caption");
    let caption = caption_node
        .map(|node| decode_blocks(path, node.children(), losses, depth))
        .map(clean_caption);

    // If work type is still none, attempt to infer from the caption (often
    // there will be no label, only a caption)
    if work_type.is_none()
        && let Some(caption) = &caption
    {
        let caption = to_text(caption);
        work_type = work_type_of(&strip_supplement_prefix(caption.trim()).to_lowercase());
    }

    // If work type is still none, attempt to infer from the media type stated
    // by the resource, or by the element itself, or of the file resolved,
    // and then from the format of the target
    if work_type.is_none()
        && let Some(media_type) = resource
            .as_ref()
            .and_then(media_type)
            .or_else(|| media_type(node))
            .or_else(|| resolved.and_then(|(_, media_type)| media_type))
    {
        work_type = match media_type.split('/').next() {
            Some("audio") => Some(CreativeWorkType::AudioObject),
            Some("image") => Some(CreativeWorkType::ImageObject),
            Some("video") => Some(CreativeWorkType::VideoObject),
            _ => None,
        };
    }
    if work_type.is_none() {
        if format.is_audio() {
            work_type = Some(CreativeWorkType::AudioObject);
        } else if format.is_image() {
            work_type = Some(CreativeWorkType::ImageObject);
        } else if format.is_video() {
            work_type = Some(CreativeWorkType::VideoObject);
        } else if target_is_datatable {
            work_type = Some(CreativeWorkType::Datatable);
        }
    }

    // Any other children (e.g. a description, attribution or further
    // resources) have nowhere to go
    for child in node.children() {
        if Some(child) != resource && Some(child) != label_node && Some(child) != caption_node {
            record_node_lost(path, &child, losses);
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

/// Infer the type of a supplementary work from the start of its label or
/// caption, in lowercase
fn work_type_of(text: &str) -> Option<CreativeWorkType> {
    [
        ("figure", CreativeWorkType::Figure),
        ("table", CreativeWorkType::Table),
        ("audio", CreativeWorkType::AudioObject),
        ("image", CreativeWorkType::ImageObject),
        ("video", CreativeWorkType::VideoObject),
        ("dataset", CreativeWorkType::Dataset),
    ]
    .into_iter()
    .find_map(|(prefix, work_type)| text.starts_with(prefix).then_some(work_type))
}

/// Strip a leading word such as "Supplementary" from a label or caption
///
/// Only whole words are stripped, ignoring case, so that a label that merely
/// starts with the same letters (e.g. "Supplemental" in "Supplementation") is
/// left as it is.
fn strip_supplement_prefix(text: &str) -> &str {
    const PREFIXES: &[&str] = &["supplementary", "supplemental", "supplement", "supporting"];

    let text = text.trim();
    PREFIXES
        .iter()
        .find_map(|prefix| {
            let head = text.get(..prefix.len())?;
            let rest = &text[prefix.len()..];
            (head.eq_ignore_ascii_case(prefix)
                && rest
                    .chars()
                    .next()
                    .is_none_or(|char| !char.is_alphanumeric()))
            .then(|| rest.trim_start())
        })
        .unwrap_or(text)
}

/// Decode a `<fig-group>` element to a vector of Stencila [`Block::Figure`]s
fn decode_fig_group(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Vec<Block> {
    record_attrs_lost(path, node, [], losses);

    decode_blocks(path, node.children(), losses, depth)
}

/// Decode the DOI of a `<fig>` or `<table-wrap>` from its `<object-id>` children
///
/// Publishers such as PLOS give each figure and table its own DOI, which is the
/// only identifier of the object that is addressable outside the article. Other
/// kinds of `<object-id>` are source-system slugs which duplicate the element's
/// `id`, so they are recorded as lost rather than retained.
fn decode_object_id(path: &str, node: &Node, losses: &mut DecodeContext) -> Option<String> {
    let mut doi = None;
    for child in node
        .children()
        .filter(|child| child.tag_name().name() == "object-id")
    {
        if doi.is_none()
            && child.attribute("pub-id-type") == Some("doi")
            && let Some(value) = child.text().map(str::trim).filter(|doi| !doi.is_empty())
        {
            record_attrs_lost(
                &extend_path(path, "object-id"),
                &child,
                ["pub-id-type"],
                losses,
            );
            doi = Some(value.to_string());
        } else {
            record_node_lost(path, &child, losses);
        }
    }

    doi
}

/// Decode a `<fig>` element to a Stencila [`Block::Figure`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/fig.html
fn decode_fig(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Block {
    let id = node.attribute("id").map(String::from);

    record_attrs_lost(path, node, ["id"], losses);

    let doi = decode_object_id(path, node, losses);

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| non_empty_text_deep(&node))
        .map(|label| clean_label(&label, FIGURE_PREFIXES));

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
            tag_name != "label" && tag_name != "caption" && tag_name != "object-id"
        }),
        losses,
        depth,
    );

    Block::Figure(Figure {
        id,
        doi,
        content,
        caption,
        label_automatically,
        label,
        ..Default::default()
    })
}

/// Decode a `<fn>` element in a block context
///
/// A footnote is a [`Note`], which in the Stencila Schema is an inline, so the
/// note is wrapped in a paragraph. Encoding that paragraph emits `<p><fn>…</fn></p>`
/// which decodes back to the same node, keeping the note addressable by its id
/// and keeping its type and content intact.
fn decode_fn(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Vec<Block> {
    let id = node.attribute("id").map(String::from);
    let note_type = note_type(node);

    record_attrs_lost(path, node, ["id", "fn-type", "custom-type"], losses);

    let content = decode_fn_content(path, node, losses, depth);
    if content.is_empty() {
        return Vec::new();
    }

    vec![p([Inline::Note(Note {
        id,
        note_type,
        content,
        ..Default::default()
    })])]
}

/// Decode the content of a `<fn>` element, ignoring its `<label>`
///
/// A footnote label is a marker (e.g. `1`, `*`) that is regenerated on display,
/// and there is no property for it on [`Note`], so it is recorded as lost.
fn decode_fn_content(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Vec<Block> {
    let mut children = Vec::new();
    for child in node.children() {
        if child.tag_name().name() == "label" {
            losses.add(extend_path(path, "label"));
        } else {
            children.push(child);
        }
    }

    decode_blocks(path, children.into_iter(), losses, depth)
}

/// Determine the [`NoteType`] of a `<fn>` element from its `fn-type` and `custom-type`
fn note_type(node: &Node) -> NoteType {
    let fn_type = node.attribute("fn-type").unwrap_or_default();
    let custom_type = node.attribute("custom-type").unwrap_or_default();

    if fn_type == "custom" {
        match custom_type.to_lowercase().as_str() {
            "endnote" => NoteType::Endnote,
            "sidenote" => NoteType::Sidenote,
            _ => NoteType::Footnote,
        }
    } else {
        NoteType::Footnote
    }
}

/// Decode a `<fn-group>` element to a vector of Stencila [`Block`]s
///
/// Some footnote groups are section-like: competing interests and author
/// contribution statements are prose that happens to be tagged as footnotes.
/// Those become a typed [`Section`] holding the statements directly. Any other
/// group is a transparent wrapper around its footnotes, which are preserved as
/// [`Note`] nodes.
fn decode_fn_group(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Vec<Block> {
    let section_type = section_type(path, node, "content-type", [], losses);

    let Some(section_type) = section_type else {
        return decode_blocks(path, node.children(), losses, depth);
    };

    // Take the content of each footnote directly: within a section-like group
    // the notes are the statements of the section, not annotations on it
    let mut content = Vec::new();
    for child in node.children() {
        let tag = child.tag_name().name();
        if tag == "fn" {
            let child_path = extend_path(path, tag);
            record_attrs_lost(&child_path, &child, [], losses);
            content.append(&mut decode_fn_content(&child_path, &child, losses, depth));
        } else {
            content.append(&mut decode_blocks(path, once(child), losses, depth));
        }
    }

    vec![Block::Section(Section {
        section_type: Some(section_type),
        content,
        ..Default::default()
    })]
}

/// Decode a `<graphic>` element to an [`ImageObject`]
fn decode_graphic(path: &str, node: &Node, losses: &mut DecodeContext) -> ImageObject {
    let (url, inferred_type) = losses.media(node);

    record_attrs_lost(path, node, ["href", "mimetype", "mime-subtype"], losses);

    ImageObject {
        content_url: url,
        media_type: media_type(node).or(inferred_type),
        caption: alt_text(node),
        ..Default::default()
    }
}

/// The media type of a `<graphic>`, `<inline-graphic>` or `<media>` element
fn media_type(node: &Node) -> Option<String> {
    let mime_type = node.attribute("mimetype")?.trim();
    if mime_type.is_empty() {
        return None;
    }

    Some(match node.attribute("mime-subtype").map(str::trim) {
        Some(subtype) if !subtype.is_empty() => [mime_type, "/", subtype].concat(),
        _ => mime_type.to_string(),
    })
}

/// The `<alt-text>` of a media element, as a caption
fn alt_text(node: &Node) -> Option<Vec<Inline>> {
    let text = node
        .children()
        .find(|child| child.has_tag_name("alt-text"))
        .and_then(|child| non_empty_text_deep(&child))?;

    Some(vec![t(text)])
}

/// Select the alternative of an `<alternatives>` element to decode
///
/// The alternatives are the same content in different forms, so only one is
/// decoded: the first whose element is one of those `preferred`, in that
/// order, and otherwise the first. The others are not lost.
fn select_alternative<'a, 'input>(
    node: &Node<'a, 'input>,
    preferred: &[&str],
) -> Option<Node<'a, 'input>> {
    preferred
        .iter()
        .find_map(|tag| node.children().find(|child| child.has_tag_name(*tag)))
        .or_else(|| node.children().find(Node::is_element))
}

/// Decode a `<code>` to a Stencila [`Block::CodeBlock`] or Stencila [`Block::CodeChunk`]
///
/// Only code that is `executable="yes"` becomes a chunk: JATS uses `no` for
/// code that can not, or should not, be executed.
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/code.html
fn decode_code(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Block {
    let id = node.attribute("id").map(String::from);
    let code = code_text(node);
    let programming_language = node.attribute("language").map(String::from);

    let executable = node.attribute("executable");
    if executable == Some("yes") {
        let label_type = node
            .attribute("label-type")
            .and_then(|string| string.parse().ok());

        record_attrs_lost(
            path,
            node,
            ["id", "language", "executable", "label-type"],
            losses,
        );

        let label = node
            .children()
            .find(|child| child.tag_name().name() == "label")
            .and_then(|node| non_empty_text_deep(&node));

        let label_automatically = label.is_some().then_some(false);

        let caption = node
            .children()
            .find(|child| child.tag_name().name() == "caption")
            .map(|node| decode_blocks(path, node.children(), losses, depth))
            .map(clean_caption);

        return Block::CodeChunk(CodeChunk {
            id,
            code,
            programming_language,
            caption,
            label,
            label_type,
            label_automatically,
            ..Default::default()
        });
    }

    // A value other than `yes` or `no` says something that is not represented
    let not_lost = match executable {
        Some("no") | None => ["id", "language", "executable"].as_slice(),
        _ => ["id", "language"].as_slice(),
    };
    record_attrs_lost(path, node, not_lost.iter().copied(), losses);

    Block::CodeBlock(CodeBlock {
        id,
        code,
        programming_language,
        ..Default::default()
    })
}

/// Get the code of a `<code>` element
///
/// Concatenates all of the text within the element, so that code split by a
/// comment, or with formatting within it, is not truncated. The text of a
/// `<label>` or `<caption>` is not code.
fn code_text(node: &Node) -> Cord {
    node.descendants()
        .filter(|descendant| {
            descendant.is_text()
                && !descendant
                    .ancestors()
                    .take_while(|ancestor| ancestor != node)
                    .any(|ancestor| {
                        ancestor.has_tag_name("label") || ancestor.has_tag_name("caption")
                    })
        })
        .filter_map(|descendant| descendant.text())
        .collect::<String>()
        .into()
}

/// Decode a `<disp-formula>` to a Stencila [`Block::MathBlock`]
///
/// see https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/disp-formula.html
fn decode_disp_formula(path: &str, node: &Node, losses: &mut DecodeContext) -> Block {
    let id = node.attribute("id").map(String::from);

    let FormulaMath {
        code,
        math_language,
        mathml,
    } = decode_formula_math(path, node, losses);

    record_attrs_lost(path, node, ["id", "code", "language"], losses);

    let label = node
        .children()
        .find(|child| child.tag_name().name() == "label")
        .and_then(|node| non_empty_text_deep(&node))
        .map(|label| clean_math_block_label(&label));

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
            mathml,
            images,
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// The math of a `<disp-formula>` or `<inline-formula>`
struct FormulaMath {
    code: Cord,
    math_language: Option<String>,
    mathml: Option<String>,
}

/// Decode the math of a formula
///
/// JATS states math as MathML, as TeX, or as both within `<alternatives>`, so
/// both are searched for among the descendants. TeX is preferred as the `code`
/// because it is what an author wrote; MathML is kept as well, in `mathml`, so
/// that it does not have to be derived from the TeX again.
///
/// A formula may also state its math in pieces, as more than one MathML
/// element with text between them, which are combined so that none of them
/// are dropped. A formula with neither MathML nor TeX may state its math as
/// text with formatting such as superscripts, which is converted to TeX.
fn decode_formula_math(path: &str, node: &Node, losses: &mut DecodeContext) -> FormulaMath {
    // Stencila's own attributes, which are only present when the JATS was
    // encoded from a math language that JATS has no element for
    let code = node.attribute("code").unwrap_or_default().to_string();
    let math_language = node.attribute("language").map(String::from);
    if !code.is_empty() {
        return FormulaMath {
            code: code.into(),
            math_language,
            mathml: None,
        };
    }

    // The pieces of a formula that states its math as more than one MathML
    // element, or as MathML with text alongside it
    let pieces = node
        .children()
        .filter(|child| {
            child.has_tag_name("math")
                || child
                    .text()
                    .is_some_and(|text| child.is_text() && !text.trim().is_empty())
        })
        .collect_vec();
    if pieces.len() > 1
        && pieces.iter().any(Node::is_element)
        && let Some(mathml) = combine_mathml(&pieces)
    {
        return FormulaMath {
            code: mathml.into(),
            math_language: Some("mathml".into()),
            mathml: None,
        };
    }

    let descendant = |name: &str| {
        node.descendants()
            .find(|child| child.is_element() && child.tag_name().name() == name)
    };

    let mathml = descendant("math").and_then(|node| serialize_node(node).ok());
    let tex = descendant("tex-math").and_then(|node| non_empty_text_deep(&node));

    match (tex, mathml) {
        (Some(tex), mathml) => FormulaMath {
            code: tex.into(),
            math_language: Some("tex".into()),
            mathml,
        },
        // MathML that is the only statement of the math is the code itself,
        // rather than a representation compiled from some other language
        (None, Some(mathml)) => FormulaMath {
            code: mathml.into(),
            math_language: Some("mathml".into()),
            mathml: None,
        },
        (None, None) => {
            let mut tex = String::new();
            text_to_tex(path, node, &mut tex, losses);
            let tex = tex.trim();
            if tex.is_empty() {
                FormulaMath {
                    code: Cord::default(),
                    math_language,
                    mathml: None,
                }
            } else {
                FormulaMath {
                    code: tex.into(),
                    math_language: Some("tex".into()),
                    mathml: None,
                }
            }
        }
    }
}

/// Combine the pieces of a formula into a single MathML element
///
/// The content of each MathML piece, and any text between them, become one
/// row of the combined element.
fn combine_mathml(pieces: &[Node]) -> Option<String> {
    const MATHML: &str = "http://www.w3.org/1998/Math/MathML";

    let mut row = String::new();
    for piece in pieces {
        if piece.is_text() {
            let text = piece.text()?.trim();
            row.push_str(&format!("<mml:mtext>{}</mml:mtext>", escape(text)));
        } else {
            for child in piece.children().filter(Node::is_element) {
                row.push_str(&serialize_node(child).ok()?);
            }
        }
    }

    Some(format!(
        "<mml:math xmlns:mml=\"{MATHML}\"><mml:mrow>{row}</mml:mrow></mml:math>"
    ))
}

/// Convert the text, and formatting, of a formula to TeX
///
/// Formatting with no equivalent in TeX math has its text kept, and is
/// recorded as lost. Labels and graphics are not part of the math.
fn text_to_tex(path: &str, node: &Node, tex: &mut String, losses: &mut DecodeContext) {
    for child in node.children() {
        if child.is_text() {
            tex.push_str(&escape_tex(child.text().unwrap_or_default()));
            continue;
        }
        if !child.is_element() {
            continue;
        }

        let tag = child.tag_name().name();
        let child_path = extend_path(path, tag);
        let (before, after) = match tag {
            "label" | "graphic" | "inline-graphic" | "alternatives" => continue,
            "italic" => ("", ""),
            "bold" => ("\\mathbf{", "}"),
            "roman" => ("\\mathrm{", "}"),
            "sup" => ("^{", "}"),
            "sub" => ("_{", "}"),
            _ => {
                record_node_lost(path, &child, losses);
                ("", "")
            }
        };
        record_attrs_lost(&child_path, &child, [], losses);
        tex.push_str(before);
        text_to_tex(&child_path, &child, tex, losses);
        tex.push_str(after);
    }
}

/// Escape the characters of text that have a special meaning in TeX
fn escape_tex(text: &str) -> String {
    text.chars()
        .map(|char| match char {
            '\\' => "\\backslash ".to_string(),
            '{' | '}' | '$' | '&' | '#' | '%' | '_' => ["\\", &char.to_string()].concat(),
            '^' => "\\hat{}".to_string(),
            '~' => "\\sim ".to_string(),
            char => char.to_string(),
        })
        .collect()
}

/// Decode a `<list>` to a Stencila [`Block::List`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/list.html
fn decode_list(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Block {
    // The `ListOrder` says whether a list is ordered, but not the style of its
    // markers, so a type that says more than that (e.g. `alpha-lower`, or
    // `simple`, which has no markers) is approximated and recorded as lost
    let list_type = node.attribute("list-type");
    let (order, exact) = match list_type {
        Some("Unordered" | "bullet") => (ListOrder::Unordered, true),
        Some("Descending") => (ListOrder::Descending, true),
        None | Some("Ascending" | "order") => (ListOrder::Ascending, true),
        Some("simple") => (ListOrder::Unordered, false),
        Some(_) => (ListOrder::Ascending, false),
    };

    let mut not_lost = vec!["id"];
    if exact {
        not_lost.push("list-type");
    }
    record_attrs_lost(path, node, not_lost, losses);

    let mut items = Vec::new();
    for child in node.children() {
        let tag = child.tag_name().name();
        if tag == "list-item" {
            items.push(decode_list_item(
                &extend_path(path, tag),
                &child,
                losses,
                depth,
            ));
        } else {
            record_node_lost(path, &child, losses);
        }
    }

    Block::List(List {
        id: node.attribute("id").map(String::from),
        ..List::new(items, order)
    })
}

/// Decode a `<list-item>` to a Stencila [`ListItem`]
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/list-item.html
fn decode_list_item(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> ListItem {
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
/// The table may be stated as rows, as an image, or as both within
/// `<alternatives>`. The rows of only one `<table>` are used, so that
/// alternative forms of the same table are not concatenated, and any images
/// are kept as well as the rows.
///
/// See https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/table-wrap.html
fn decode_table_wrap(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> Block {
    let id = node.attribute("id").map(String::from);

    record_attrs_lost(path, node, ["id"], losses);

    let doi = decode_object_id(path, node, losses);

    let mut label = None;
    let mut caption = None;
    let mut tables = Vec::new();
    let mut graphics = Vec::new();
    let mut notes = None;
    for child in node.children() {
        let tag = child.tag_name().name();
        match tag {
            "label" if label.is_none() => {
                label = non_empty_text_deep(&child).map(|label| clean_label(&label, TABLE_PREFIXES))
            }
            "caption" if caption.is_none() => {
                caption = Some(clean_caption(decode_blocks(
                    &extend_path(path, tag),
                    child.children(),
                    losses,
                    depth,
                )))
            }
            "table" => tables.push(child),
            "graphic" => graphics.push(child),
            "alternatives" => {
                record_attrs_lost(&extend_path(path, tag), &child, [], losses);
                for alternative in child.children().filter(Node::is_element) {
                    match alternative.tag_name().name() {
                        "table" => tables.push(alternative),
                        "graphic" => graphics.push(alternative),
                        // Another form of the same table
                        _ => {}
                    }
                }
            }
            "table-wrap-foot" if notes.is_none() => {
                notes = Some(decode_blocks(
                    &extend_path(path, tag),
                    child.children(),
                    losses,
                    depth,
                ))
            }
            // The DOI is decoded above
            "object-id" => {}
            _ => record_node_lost(path, &child, losses),
        }
    }

    let label_automatically = label.is_some().then_some(false);

    let rows = tables
        .first()
        .map(|table| {
            let path = &extend_path(path, "table");
            table
                .children()
                .flat_map(|child| {
                    let tag = child.tag_name().name();
                    let child_path = extend_path(path, tag);
                    let row_type = match tag {
                        "thead" => Some(TableRowType::HeaderRow),
                        "tfoot" => Some(TableRowType::FooterRow),
                        _ => None,
                    };
                    if tag == "thead" || tag == "tbody" || tag == "tfoot" {
                        decode_table_section(&child_path, &child, losses, depth, row_type)
                    } else if tag == "tr" {
                        vec![decode_table_row(path, &child, losses, depth, None)]
                    } else {
                        // e.g. a `<col>` or `<colgroup>`, which describes the
                        // layout of the columns rather than their content
                        record_node_lost(path, &child, losses);
                        Vec::new()
                    }
                })
                .collect::<Vec<TableRow>>()
        })
        .unwrap_or_default();
    // A second table that is not an alternative of the first is lost
    for table in tables.iter().skip(1) {
        if !table
            .parent()
            .is_some_and(|parent| parent.has_tag_name("alternatives"))
        {
            record_node_lost(path, table, losses);
        }
    }

    let images = graphics
        .iter()
        .map(|graphic| decode_graphic(&extend_path(path, "graphic"), graphic, losses))
        .collect_vec();
    let images = (!images.is_empty()).then_some(images);

    Block::Table(Table {
        id,
        doi,
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
    losses: &mut DecodeContext,
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
    losses: &mut DecodeContext,
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
fn decode_table_cell(path: &str, node: &Node, losses: &mut DecodeContext, depth: u8) -> TableCell {
    let cell_type = if node.tag_name().name() == "th" {
        Some(TableCellType::HeaderCell)
    } else {
        None
    };

    let row_span = node
        .attribute("rowspan")
        .and_then(|alignment| alignment.parse().ok())
        .and_then(|row_span| (row_span != 1).then_some(row_span));

    let column_span = node
        .attribute("colspan")
        .and_then(|alignment| alignment.parse().ok())
        .and_then(|row_span| (row_span != 1).then_some(row_span));

    let vertical_alignment = node.attribute("valign").and_then(vertical_alignment);
    let horizontal_alignment = node.attribute("align").and_then(horizontal_alignment);

    // The character that a column of numbers is aligned on only means something
    // when the column is aligned on a character
    let horizontal_alignment_character = matches!(
        horizontal_alignment,
        Some(HorizontalAlignment::AlignCharacter)
    )
    .then(|| node.attribute("char").map(String::from))
    .flatten();

    let mut not_lost = vec!["rowspan", "colspan"];
    if vertical_alignment.is_some() {
        not_lost.push("valign");
    }
    if horizontal_alignment.is_some() {
        not_lost.push("align");
    }
    if horizontal_alignment_character.is_some() {
        not_lost.push("char");
    }
    record_attrs_lost(path, node, not_lost, losses);

    // A cell can mix inline runs with blocks. Decode each run as a paragraph
    // and each block directly, keeping their order and avoiding trial decodes
    // that would incorrectly report supported children as lost.
    let mut content = Vec::new();
    let mut inlines = Vec::new();
    for child in node.children() {
        if child.is_element() && is_cell_block(child.tag_name().name()) {
            if let Some(paragraph) =
                decode_cell_paragraph(path, std::mem::take(&mut inlines), losses)
            {
                content.push(paragraph);
            }
            content.extend(decode_blocks(path, once(child), losses, depth));
        } else {
            inlines.push(child);
        }
    }
    if let Some(paragraph) = decode_cell_paragraph(path, inlines, losses) {
        content.push(paragraph);
    }

    TableCell {
        cell_type,
        content,
        options: Box::new(TableCellOptions {
            row_span,
            column_span,
            vertical_alignment,
            horizontal_alignment,
            horizontal_alignment_character,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Decode an inline run within a table cell, trimming only whitespace text.
///
/// An image or another non-text node can have an empty text representation
/// while still carrying content, so it must never be trimmed on that basis.
fn decode_cell_paragraph(
    path: &str,
    nodes: Vec<Node>,
    losses: &mut DecodeContext,
) -> Option<Block> {
    let mut inlines = decode_inlines(path, nodes.into_iter(), losses);
    let is_content = |inline: &Inline| !matches!(inline, Inline::Text(Text { value, .. }) if value.trim().is_empty());
    let start = inlines.iter().position(is_content)?;
    let end = inlines.iter().rposition(is_content)?;
    Some(p(inlines.drain(start..=end).collect_vec()))
}

/// Decode the JATS `align` attribute of a table cell
///
/// The schema names each alignment after what it does rather than after the
/// JATS value, so the two are mapped rather than parsed.
fn horizontal_alignment(value: &str) -> Option<HorizontalAlignment> {
    Some(match value.trim().to_lowercase().as_str() {
        "left" => HorizontalAlignment::AlignLeft,
        "right" => HorizontalAlignment::AlignRight,
        "center" => HorizontalAlignment::AlignCenter,
        "justify" => HorizontalAlignment::AlignJustify,
        "char" => HorizontalAlignment::AlignCharacter,
        _ => return None,
    })
}

/// Decode the JATS `valign` attribute of a table cell
fn vertical_alignment(value: &str) -> Option<VerticalAlignment> {
    Some(match value.trim().to_lowercase().as_str() {
        "top" => VerticalAlignment::AlignTop,
        "middle" => VerticalAlignment::AlignMiddle,
        "bottom" => VerticalAlignment::AlignBottom,
        "baseline" => VerticalAlignment::AlignBaseline,
        _ => return None,
    })
}

/// Decode inline content nodes
///
/// Iterates over all child elements and either decodes them, or adds them to
/// losses.
pub fn decode_inlines<'a, 'input: 'a, I: Iterator<Item = Node<'a, 'input>>>(
    path: &str,
    nodes: I,
    losses: &mut DecodeContext,
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
                "graphic" => Inline::ImageObject(decode_graphic(&child_path, &child, losses)),
                "inline-formula" => decode_inline_formula(&child_path, &child, losses),
                "alternatives" => {
                    record_attrs_lost(&child_path, &child, [], losses);
                    if let Some(alternative) = select_alternative(
                        &child,
                        &[
                            "inline-graphic",
                            "inline-formula",
                            "inline-media",
                            "graphic",
                        ],
                    ) {
                        inlines.append(&mut decode_inlines(&child_path, once(alternative), losses));
                    }
                    continue;
                }
                "uri" => decode_uri(&child_path, &child, losses),
                "inline-graphic" | "inline-media" | "media" => {
                    decode_inline_media(&child_path, &child, losses)
                }
                "math" => decode_inline_math(&child),
                "parameter" => decode_parameter(&child_path, &child, losses),
                "styled-content" => decode_styled_content(&child_path, &child, losses),
                "email" => decode_email(&child_path, &child, losses),
                // A line break within a run of text separates the words on
                // either side of it, which is all that the schema records here
                "break" => {
                    record_attrs_lost(&child_path, &child, [], losses);
                    t(" ")
                }
                "time" => decode_time(&child_path, &child, losses),
                "timestamp" => decode_timestamp(&child_path, &child, losses),
                "xref" => match xref_type(&child, losses) {
                    Some(JatsRefType::Bibr | JatsRefType::Ref) => {
                        decode_xref_citation(&child_path, &child, losses)
                    }
                    Some(ref_type) if ref_type.is_block_reference() => {
                        decode_xref_block(&child_path, &child, losses)
                    }
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
                        // A wrapper the schema has no inline for still says
                        // something with the words inside it, so the wrapper
                        // alone is reported and its content is kept
                        _ => {
                            record_node_lost(path, &child, losses);

                            // The parts of a citation are bibliographic fields
                            // rather than prose, so what is lost is the
                            // citation as a whole rather than each of them
                            let mut discarded = DecodeContext::default();
                            let inner = if is_citation_element(tag) {
                                &mut discarded
                            } else {
                                &mut *losses
                            };
                            inlines.append(&mut decode_inlines(&child_path, grandchildren, inner));
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
fn decode_inline_media(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    let (content_url, inferred_type) = losses.media(node);
    let media_type = media_type(node).or(inferred_type);
    let mime_type = media_type
        .as_deref()
        .and_then(|value| value.split('/').next());

    record_attrs_lost(path, node, ["href", "mimetype", "mime-subtype"], losses);

    let mut caption: Option<Vec<Inline>> = None;
    let mut description = None;
    for child in node.children() {
        let tag = child.tag_name().name();
        match tag {
            "alt-text" => caption = alt_text(node),
            "long-desc" => description = non_empty_text_deep(&child),
            _ => record_node_lost(path, &child, losses),
        }
    }

    if node.tag_name().name() == "inline-graphic" || mime_type == Some("image") {
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

    match mime_type {
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
fn decode_inline_code(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    let executable = node.attribute("executable").map(String::from);
    let programming_language = node.attribute("language").map(String::from);
    let code = code_text(node);

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
fn decode_date(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
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
fn decode_date_time(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
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
fn decode_duration(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
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
///
/// The JATS-specific `ext-link-type` is retained separately from HTML `rel`
/// semantics so that both can round-trip independently.
fn decode_link(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    let target = node
        .attribute((XLINK, "href"))
        .map(String::from)
        .unwrap_or_default();

    record_attrs_lost(path, node, ["id", "href", "title", "ext-link-type"], losses);

    let content = decode_inlines(path, node.children(), losses);

    // `uri` says no more than the target itself does, and is emitted again for
    // any target with a scheme, so retaining it would add a link type to every
    // link that did not have one
    let jats_ext_link_type = node
        .attribute("ext-link-type")
        .filter(|link_type| !(*link_type == "uri" && has_uri_scheme(&target)))
        .map(String::from);

    Inline::Link(Link {
        id: node.attribute("id").map(String::from),
        title: node.attribute((XLINK, "title")).map(String::from),
        target,
        jats_ext_link_type,
        content,
        ..Default::default()
    })
}

/// Decode a `<uri>` to a [`Inline::Link`]
///
/// The target is the `xlink:href` if there is one, and otherwise the text of
/// the element, which is then also the content of the link.
fn decode_uri(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    record_attrs_lost(path, node, ["href"], losses);

    let content = decode_inlines(path, node.children(), losses);
    let target = node
        .attribute((XLINK, "href"))
        .map(str::trim)
        .filter(|href| !href.is_empty())
        .map(String::from)
        .unwrap_or_else(|| to_text(&content).trim().to_string());

    Inline::Link(Link {
        target,
        content,
        ..Default::default()
    })
}

/// The type of what an `<xref>` refers to
///
/// The `ref-type` is optional in JATS, so when it is absent the type is
/// inferred from the element that the (first) `rid` refers to. An element that
/// is not the target of any type of cross-reference that is decoded (e.g. an
/// `<aff>`), or that is not in the document, has no type.
fn xref_type(node: &Node, losses: &DecodeContext) -> Option<JatsRefType> {
    if let Some(ref_type) = node.attribute("ref-type") {
        return Some(JatsRefType::from(ref_type));
    }

    let rid = node.attribute("rid")?.split_whitespace().next()?;
    let tag = losses.tag_of(rid)?;

    use JatsRefType::*;
    [
        Bibr,
        Ack,
        App,
        BoxedText,
        DispFormula,
        Fig,
        Fn,
        List,
        Media,
        Sec,
        Statement,
        SupplementaryMaterial,
        Table,
    ]
    .into_iter()
    .find(|ref_type| ref_type.target_elements().contains(&tag))
    .or_else(|| {
        (!matches!(
            tag,
            "aff" | "contrib" | "corresp" | "kwd" | "award-group" | "article" | "sub-article"
        ))
        .then_some(Other)
    })
}

/// Decode a `<fn>` to a [`Inline::Footnote`]
fn decode_footnote(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    let id = node.attribute("id").map(String::from);
    let note_type = note_type(node);

    record_attrs_lost(path, node, ["id", "fn-type", "custom-type"], losses);

    let content = decode_fn_content(path, node, losses, 0);

    Inline::Note(Note {
        id,
        note_type,
        content,
        ..Default::default()
    })
}

/// Decode a `<inline-formula>` to a [`Inline::MathInline`]
fn decode_inline_formula(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    let id = node.attribute("id").map(String::from);
    let FormulaMath {
        code,
        math_language,
        mathml,
    } = decode_formula_math(path, node, losses);

    record_attrs_lost(path, node, ["id", "code", "language"], losses);

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
        id,
        code,
        math_language,
        options: Box::new(MathInlineOptions {
            mathml,
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
fn decode_parameter(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    let name = node.attribute("name").map(String::from).unwrap_or_default();

    record_attrs_lost(path, node, ["name"], losses);

    Inline::Parameter(Parameter {
        name,
        ..Default::default()
    })
}

/// Decode a `<styled-content>` to a [`Inline::StyledInline`]
fn decode_styled_content(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
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

/// Whether an element holds the fields of a bibliographic citation
fn is_citation_element(tag: &str) -> bool {
    matches!(
        tag,
        "citation"
            | "element-citation"
            | "mixed-citation"
            | "nlm-citation"
            | "person-group"
            | "product"
            | "related-object"
    )
}

/// Decode an `<email>` to a [`Inline::Link`] that addresses the email address
fn decode_email(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    record_attrs_lost(path, node, [], losses);

    let content = decode_inlines(path, node.children(), losses);
    let address = to_text(&content);

    Inline::Link(Link {
        target: ["mailto:", address.trim()].concat(),
        content,
        ..Default::default()
    })
}

/// Decode a `<time>` to a [`Inline::Time`]
fn decode_time(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
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
fn decode_timestamp(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
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
///
/// There is no builtin way to differentiate between narrative and parenthetical
/// citations in JATS (it needs to be inferred from surrounding punctuation
/// during structuring). So in this function, do not assume one or other.
///
/// A `rid` can list more than one reference (e.g. for "[1,2]"). Because the
/// target of a citation is a single reference, such a `<xref>` is decoded to a
/// [`Inline::CitationGroup`] with a citation of each, and the content of the
/// `<xref>` as the content of the group.
fn decode_xref_citation(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    let targets = node
        .attribute("rid")
        .map(|rid| rid.split_whitespace().map(String::from).collect_vec())
        .unwrap_or_default();

    record_attrs_lost(path, node, ["ref-type", "rid"], losses);

    let content = decode_inlines(path, node.children(), losses);
    let content = (!content.is_empty()).then_some(content);

    if targets.len() > 1 {
        return Inline::CitationGroup(CitationGroup {
            items: targets.into_iter().map(Citation::new).collect(),
            content,
            ..Default::default()
        });
    }

    Inline::Citation(Citation {
        target: targets.into_iter().next().unwrap_or_default(),
        options: Box::new(CitationOptions {
            content,
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Decode a `<xref>` to a [`Inline::Link`] with a target to an internal block
///
/// A `rid` can list more than one target, as it does for a link that stands for
/// a range of figures, so each of them becomes a fragment of the link target.
/// The `ref-type` is retained as JATS-specific metadata so that a type which
/// says more than the kind of the target node, such as `table-fn`, is not lost.
fn decode_xref_block(path: &str, node: &Node, losses: &mut DecodeContext) -> Inline {
    let target = node
        .attribute("rid")
        .map(|rid| {
            rid.split_whitespace()
                .map(|id| ["#", id].concat())
                .join(" ")
        })
        .unwrap_or_default();

    let jats_ref_type = node.attribute("ref-type").map(String::from);

    record_attrs_lost(path, node, ["id", "ref-type", "rid", "title"], losses);

    let content = decode_inlines(path, node.children(), losses);

    Inline::Link(Link {
        id: node.attribute("id").map(String::from),
        target,
        title: node.attribute((XLINK, "title")).map(String::from),
        jats_ref_type,
        content,
        ..Default::default()
    })
}

/// The words that a figure label may start with, longest first
const FIGURE_PREFIXES: &[&str] = &["Figure", "figure", "FIGURE", "Fig", "fig", "FIG"];

/// The words that a table label may start with
const TABLE_PREFIXES: &[&str] = &["Table", "table", "TABLE"];

/// Clean a figure or table label by removing the first of `prefixes` that it
/// starts with, and any leading and trailing punctuation
fn clean_label(label: &str, prefixes: &[&str]) -> String {
    let cleaned = prefixes
        .iter()
        .find_map(|prefix| label.strip_prefix(prefix))
        .unwrap_or(label);

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
