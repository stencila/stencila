use std::{collections::HashMap, path::PathBuf};

use node_transform::Transform;
use pandoc_types::definition as pandoc;

use codec::{
    common::{
        eyre::{bail, Result},
        futures::executor,
        serde_json,
        slug::slugify,
    },
    stencila_schema::*,
    CodecTrait,
};
use codec_json::JsonCodec;
use codec_rpng::RpngCodec;
use codec_txt::ToTxt;
use formats::FormatNodeType;
use node_coerce::coerce;
use suids::Suid;

use crate::from_pandoc;

/// Decode a document to a `Node`
///
/// Intended primarily for use by other internal codec crates e.g. `codec-docx`, `codec-latex`
pub async fn decode(
    input: &str,
    path: Option<PathBuf>,
    format: &str,
    args: &[&str],
) -> Result<Node> {
    let pandoc = from_pandoc(input, path, format, args).await?;
    decode_pandoc(pandoc)
}

/// Decode a fragment to a vector of `BlockContent`
///
/// Intended for decoding a fragment of a larger document (e.g. from some LaTeX in
/// a Markdown document). Ignores any meta data e.g. title
pub async fn decode_fragment(
    input: &str,
    format: &str,
    args: &[&str],
) -> Result<Vec<BlockContent>> {
    if input.is_empty() {
        return Ok(vec![]);
    }

    let pandoc = from_pandoc(input, None, format, args).await?;
    let context = DecodeContext {};
    Ok(translate_blocks(&pandoc.blocks, &context))
}

/// Decode a Pandoc document to a `Node`
pub fn decode_pandoc(pandoc: pandoc::Pandoc) -> Result<Node> {
    let context = DecodeContext {};
    let mut article = translate_meta(pandoc.meta, &context)?;

    let content = translate_blocks(&pandoc.blocks, &context);
    article.content = if content.is_empty() {
        None
    } else {
        Some(content)
    };

    Ok(Node::Article(article))
}

/// Translate Pandoc meta data into an `Article` node
fn translate_meta(
    meta: HashMap<String, pandoc::MetaValue>,
    context: &DecodeContext,
) -> Result<Article> {
    let mut article = translate_meta_map(&meta, context);
    article.insert("type".to_string(), serde_json::json!("Article"));

    let node = coerce(serde_json::Value::Object(article), None)?;

    match node {
        Node::Article(article) => Ok(article),
        _ => bail!("Expected an article"),
    }
}

/// Translate a map of `MetaValue` to a map of `serde_json::Value`
fn translate_meta_map(
    map: &HashMap<String, pandoc::MetaValue>,
    context: &DecodeContext,
) -> serde_json::Map<String, serde_json::Value> {
    map.iter()
        .map(|(key, value)| (key.clone(), translate_meta_value(value, context)))
        .collect()
}

/// Translate a meta value to a `serde_json::Value`
fn translate_meta_value(value: &pandoc::MetaValue, context: &DecodeContext) -> serde_json::Value {
    match value {
        pandoc::MetaValue::MetaMap(map) => {
            serde_json::Value::Object(translate_meta_map(map, context))
        }
        pandoc::MetaValue::MetaList(vec) => serde_json::Value::Array(
            vec.iter()
                .map(|value| translate_meta_value(value, context))
                .collect(),
        ),
        pandoc::MetaValue::MetaBool(bool) => serde_json::Value::Bool(*bool),
        pandoc::MetaValue::MetaString(string) => serde_json::Value::String(string.clone()),
        pandoc::MetaValue::MetaInlines(inlines) => serde_json::Value::Array(
            translate_inlines(inlines, context)
                .iter()
                .map(|inline| serde_json::to_value(inline).expect("Can serialize to JSON value"))
                .collect(),
        ),
        pandoc::MetaValue::MetaBlocks(blocks) => serde_json::Value::Array(
            translate_blocks(blocks, context)
                .iter()
                .map(|block| serde_json::to_value(block).expect("Can serialize to JSON value"))
                .collect(),
        ),
    }
}

/// Decoding context
struct DecodeContext {}

/// Translate a vector of Pandoc `Block` elements to a vector of `BlockContent` nodes
fn translate_blocks(elements: &[pandoc::Block], context: &DecodeContext) -> Vec<BlockContent> {
    elements
        .iter()
        .flat_map(|child| translate_block(child, context))
        .collect()
}

/// Translate a Pandoc `Block` element into a zero or more `BlockContent` nodes
///
/// Will ignore elements that are dealt with by `translate_inline`
fn translate_block(element: &pandoc::Block, context: &DecodeContext) -> Vec<BlockContent> {
    match element {
        pandoc::Block::Header(depth, attrs, inlines) => {
            let content = translate_inlines(inlines, context);

            // For some formats e.g. DOCX, LaTeX. Pandoc automatically adds an `id` to headings based
            // on its content. That is useful, but redundant when it is decoded back. So, if the id
            // is a slug of the content then ignore it.
            let id = if let Some(id) = get_id(attrs) {
                let slug = slugify(content.to_txt());
                if *id == slug {
                    None
                } else {
                    Some(id)
                }
            } else {
                None
            };

            vec![BlockContent::Heading(Heading {
                id,
                depth: Some(*depth as u8),
                content,
                ..Default::default()
            })]
        }

        pandoc::Block::Para(inlines) => {
            let content = translate_inlines(inlines, context);
            if content.len() == 1 {
                if let Some(block) = transform_to_block(&content[0]) {
                    return vec![block];
                }
                if let InlineContent::MathFragment(MathFragment { code, .. }) = &content[0] {
                    return vec![BlockContent::MathBlock(MathBlock {
                        code: code.to_owned(),
                        math_language: "tex".to_string(),
                        ..Default::default()
                    })];
                }
            }
            vec![BlockContent::Paragraph(Paragraph {
                content,
                ..Default::default()
            })]
        }

        pandoc::Block::BlockQuote(blocks) => {
            vec![BlockContent::QuoteBlock(QuoteBlock {
                content: translate_blocks(blocks, context),
                ..Default::default()
            })]
        }

        pandoc::Block::CodeBlock(attrs, code) => {
            let id = get_id(attrs);
            let programming_language = get_attr(attrs, "classes").map(Box::new);
            vec![BlockContent::CodeBlock(CodeBlock {
                id,
                programming_language,
                code: code.clone(),
                ..Default::default()
            })]
        }

        pandoc::Block::OrderedList(.., items) | pandoc::Block::BulletList(items) => {
            let order = Some(match element {
                pandoc::Block::OrderedList(..) => ListOrder::Ascending,
                _ => ListOrder::Unordered,
            });
            let items = items
                .iter()
                .map(|blocks| {
                    let blocks = translate_blocks(blocks, context);
                    let content = if blocks.is_empty() {
                        None
                    } else {
                        Some(ListItemContent::VecBlockContent(blocks))
                    };
                    ListItem {
                        content,
                        ..Default::default()
                    }
                })
                .collect();
            vec![BlockContent::List(List {
                items,
                order,
                ..Default::default()
            })]
        }
        pandoc::Block::DefinitionList(definitions) => {
            let items = definitions
                .iter()
                .map(|(term, definitions)| {
                    let term = vec![BlockContent::Paragraph(Paragraph {
                        content: translate_inlines(term, context),
                        ..Default::default()
                    })];
                    let definitions = definitions
                        .iter()
                        .flat_map(|blocks| translate_blocks(blocks, context))
                        .collect();
                    let blocks = [term, definitions].concat();
                    let content = if blocks.is_empty() {
                        None
                    } else {
                        Some(ListItemContent::VecBlockContent(blocks))
                    };
                    ListItem {
                        content,
                        ..Default::default()
                    }
                })
                .collect();
            vec![BlockContent::List(List {
                items,
                ..Default::default()
            })]
        }

        pandoc::Block::Table(pandoc::Table {
            attr,
            caption,
            head,
            bodies,
            foot,
            ..
        }) => {
            let id = get_id(attr);

            let caption = translate_blocks(&caption.long, context);
            let caption = match caption.is_empty() {
                true => None,
                false => Some(Box::new(TableCaption::VecBlockContent(caption))),
            };

            let head: Vec<TableRow> = head
                .rows
                .iter()
                .map(|row| translate_row(row, context, Some(TableRowRowType::Header)))
                .collect();
            let body: Vec<TableRow> = bodies
                .iter()
                .flat_map(|body| {
                    let intermediate_head: Vec<TableRow> = body
                        .head
                        .iter()
                        .map(|row| translate_row(row, context, Some(TableRowRowType::Header)))
                        .collect();
                    let intermediate_body: Vec<TableRow> = body
                        .body
                        .iter()
                        .map(|row| translate_row(row, context, None))
                        .collect();
                    [intermediate_head, intermediate_body].concat()
                })
                .collect();
            let foot: Vec<TableRow> = foot
                .rows
                .iter()
                .map(|row| translate_row(row, context, Some(TableRowRowType::Footer)))
                .collect();
            let rows = [head, body, foot].concat();

            vec![BlockContent::Table(TableSimple {
                rows,
                caption,
                id,
                ..Default::default()
            })]
        }

        pandoc::Block::HorizontalRule => vec![BlockContent::ThematicBreak(ThematicBreak {
            ..Default::default()
        })],

        pandoc::Block::RawBlock(_format, _content) => {
            // TODO: Attempt to decode raw content; skip if not decode-able
            vec![]
        }

        // A line block is "multiple non-breaking lines" so just flatten into a paragraph
        pandoc::Block::LineBlock(lines) => vec![BlockContent::Paragraph(Paragraph {
            content: lines
                .iter()
                .flat_map(|inlines| translate_inlines(inlines, context))
                .collect(),
            ..Default::default()
        })],

        // Element types not supported by Stencila but with child elements that are
        pandoc::Block::Div(_attrs, blocks) => translate_blocks(blocks, context),
        pandoc::Block::Plain(inlines) => vec![BlockContent::Paragraph(Paragraph {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],

        // Element types not supported by Stencila
        pandoc::Block::Null => vec![],
    }
}

/// Translate a Pandoc table row into a `TableRow`
fn translate_row(
    row: &pandoc::Row,
    context: &DecodeContext,
    row_type: Option<TableRowRowType>,
) -> TableRow {
    let cell_type = match row_type {
        Some(TableRowRowType::Header) => Some(TableCellCellType::Header),
        Some(TableRowRowType::Footer) => Some(TableCellCellType::Header),
        None => None,
    };
    let cells = row
        .cells
        .iter()
        .map(|cell| translate_cell(cell, context, cell_type.clone()))
        .collect();
    TableRow {
        cells,
        row_type,
        ..Default::default()
    }
}

/// Translate a Pandoc table cell into a `TableCell`
///
/// Pandoc always returns blocks for table cells, for example, even for
/// a single number it returns a paragraph. This is somewhat heavy weight,
/// and inconsistent with other decoders in this repo, so here we unwrap
/// a single paragraph to inlines.
fn translate_cell(
    cell: &pandoc::Cell,
    context: &DecodeContext,
    cell_type: Option<TableCellCellType>,
) -> TableCell {
    let pandoc::Cell {
        row_span,
        col_span,
        content: blocks,
        ..
    } = cell;
    let blocks = translate_blocks(blocks, context);
    let content = match blocks.len() {
        0 => None,
        1 => match &blocks[0] {
            BlockContent::Paragraph(paragraph) => Some(TableCellContent::VecInlineContent(
                paragraph.content.clone(),
            )),
            _ => Some(TableCellContent::VecBlockContent(blocks)),
        },
        _ => Some(TableCellContent::VecBlockContent(blocks)),
    };
    let rowspan = match row_span {
        1 => None,
        _ => Some((*row_span) as u32),
    };
    let colspan = match col_span {
        1 => None,
        _ => Some((*col_span) as u32),
    };
    TableCell {
        colspan,
        content,
        cell_type,
        rowspan,
        ..Default::default()
    }
}

/// Translate a vector of Pandoc `Inline` elements to a vector of `InlineContent` nodes
fn translate_inlines(elements: &[pandoc::Inline], context: &DecodeContext) -> Vec<InlineContent> {
    let mut inlines: Vec<InlineContent> = elements
        .iter()
        .flat_map(|child| translate_inline(child, context))
        .collect();

    let mut index = 1;
    while index < inlines.len() {
        let curr = inlines[index].clone();
        match (&mut inlines[index - 1], curr) {
            (InlineContent::String(prev), InlineContent::String(curr)) => {
                match curr.as_str() {
                    "\u{2029}" => prev.push(' '),
                    _ => prev.push_str(&curr),
                };
                inlines.remove(index);
            }
            _ => {
                index += 1;
            }
        }
    }

    inlines
}

/// Translate a Pandoc `Inline` element into a zero or more `InlineContent` nodes
fn translate_inline(element: &pandoc::Inline, context: &DecodeContext) -> Vec<InlineContent> {
    let inlines = match element {
        pandoc::Inline::Str(string) => vec![InlineContent::String(string.clone())],
        pandoc::Inline::Space => vec![InlineContent::String(" ".to_string())],
        pandoc::Inline::SoftBreak => vec![InlineContent::String("\u{2029}".to_string())],

        pandoc::Inline::Emph(inlines) => vec![InlineContent::Emphasis(Emphasis {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],
        pandoc::Inline::Underline(inlines) => {
            vec![InlineContent::Underline(Underline {
                content: translate_inlines(inlines, context),
                ..Default::default()
            })]
        }
        pandoc::Inline::Strong(inlines) => vec![InlineContent::Strong(Strong {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],
        pandoc::Inline::Strikeout(inlines) => vec![InlineContent::Strikeout(Strikeout {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],
        pandoc::Inline::Superscript(inlines) => vec![InlineContent::Superscript(Superscript {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],
        pandoc::Inline::Subscript(inlines) => vec![InlineContent::Subscript(Subscript {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],
        pandoc::Inline::Quoted(_quote_type, inlines) => vec![InlineContent::Quote(Quote {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],

        pandoc::Inline::Code(attrs, code) => {
            let id = get_id(attrs);
            vec![InlineContent::CodeFragment(CodeFragment {
                id,
                code: code.clone(),
                ..Default::default()
            })]
        }
        pandoc::Inline::Math(_math_type, code) => vec![InlineContent::MathFragment(MathFragment {
            code: code.clone(),
            math_language: "tex".to_string(),
            ..Default::default()
        })],

        pandoc::Inline::Link(attrs, inlines, target) => {
            let pandoc::Target { url, title } = target;
            vec![InlineContent::Link(Link {
                id: get_id(attrs),
                target: url.clone(),
                title: get_string_prop(title).map(Box::new),
                content: translate_inlines(inlines, context),
                ..Default::default()
            })]
        }
        pandoc::Inline::Image(attrs, inlines, target) => {
            let id = get_id(attrs);

            let caption = translate_inlines(inlines, context).to_txt();
            let caption = match caption.is_empty() {
                true => None,
                false => Some(Box::new(caption)),
            };

            let pandoc::Target { url, title } = target;
            let content_url = url.clone();
            let title = match title.is_empty() {
                true => None,
                false => Some(vec![InlineContent::String(title.to_string())]),
            };

            match formats::match_path(&content_url).spec().node_type {
                FormatNodeType::AudioObject => {
                    vec![InlineContent::AudioObject(AudioObjectSimple {
                        content_url,
                        title,
                        id,
                        ..Default::default()
                    })]
                }
                FormatNodeType::VideoObject => {
                    vec![InlineContent::VideoObject(VideoObjectSimple {
                        content_url,
                        title,
                        id,
                        ..Default::default()
                    })]
                }
                _ => vec![InlineContent::ImageObject(ImageObjectSimple {
                    content_url,
                    title,
                    caption,
                    id,
                    ..Default::default()
                })],
            }
        }

        pandoc::Inline::Cite(citations, _inlines) => {
            let items: Vec<Cite> = citations
                .iter()
                .map(|citation| {
                    // TODO: Use inlines and prefix and suffix (consider using Vec<InlineContent> for both
                    // and parsing out pagination)
                    let citation_mode = Some(match citation.citation_mode {
                        pandoc::CitationMode::NormalCitation => CiteCitationMode::Parenthetical,
                        pandoc::CitationMode::AuthorInText => CiteCitationMode::Narrative,
                        pandoc::CitationMode::SuppressAuthor => CiteCitationMode::NarrativeYear,
                    });
                    Cite {
                        citation_mode,
                        target: citation.citation_id.clone(),
                        ..Default::default()
                    }
                })
                .collect();
            match items.len() {
                0 => vec![],
                1 => vec![InlineContent::Cite(items[0].clone())],
                _ => vec![InlineContent::CiteGroup(CiteGroup {
                    items,
                    ..Default::default()
                })],
            }
        }

        pandoc::Inline::Note(_blocks) => {
            // TODO: Decode blocks to the context and link to their Stencila `Note`
            vec![]
        }

        pandoc::Inline::RawInline(_format, _content) => {
            // TODO: Attempt to decode raw content; skip if not decode-able
            vec![]
        }

        // Element types not supported by Stencila but with child elements that are
        pandoc::Inline::SmallCaps(inlines) | pandoc::Inline::Span(.., inlines) => {
            translate_inlines(inlines, context)
        }

        // Element types not supported by Stencila
        pandoc::Inline::LineBreak => vec![],
    };

    // Try to transform inline nodes as needed
    inlines
        .into_iter()
        .map(|inline| transform_to_inline(&inline).unwrap_or(inline))
        .collect()
}

/// Get an attribute from a Pandoc `Attr` tuple struct
fn get_attr(attrs: &pandoc::Attr, name: &str) -> Option<String> {
    match name {
        "id" => match attrs.identifier.is_empty() {
            true => None,
            false => Some(attrs.identifier.clone()),
        },
        "classes" => match attrs.classes.is_empty() {
            true => None,
            false => Some(attrs.classes.join(" ")),
        },
        _ => attrs.attributes.iter().find_map(|(key, value)| {
            if key == name {
                Some(value.clone())
            } else {
                None
            }
        }),
    }
}

/// Get an optional string property from a string
/// If the string is empty, then will be `None`
fn get_string_prop(value: &str) -> Option<String> {
    match value.is_empty() {
        true => None,
        false => Some(value.to_string()),
    }
}

// Get the `id` property of a `Entity` from a  Pandoc `Attr` tuple struct
fn get_id(attrs: &pandoc::Attr) -> Option<Suid> {
    get_attr(attrs, "id")
        .and_then(|value| get_string_prop(&value))
        .map(|id| id.into())
}

/// Try to transform inline content (potentially containing an RPNG) to another type of inline content
fn transform_to_inline(inline: &InlineContent) -> Option<InlineContent> {
    match inline {
        InlineContent::Link(link) => {
            if link.content.len() == 1 {
                // Try to get a node from the inner content
                if let Some(inline) = transform_to_inline(&link.content[0]) {
                    return Some(inline);
                }
            }
            // Fallback to fetching node from the link's URL
            // TODO
            None
        }
        InlineContent::ImageObject(image) => {
            // Try to get the node from the caption
            if let Some(caption) = image.caption.as_deref() {
                if let Ok(node) = JsonCodec::from_str(caption, None) {
                    if node.is_inline() {
                        return Some(node.to_inline());
                    }
                }
            }
            // Fallback to getting from the image, either a data URI, or a file
            let url = &image.content_url;
            if url.starts_with("data:image/png;") {
                if let Ok(node) = RpngCodec::from_str(url, None) {
                    if node.is_inline() {
                        return Some(node.to_inline());
                    }
                }
            }
            let file = PathBuf::from(url);
            if file.exists() {
                if let Ok(node) = executor::block_on(RpngCodec::from_path(&file, None)) {
                    if node.is_inline() {
                        return Some(node.to_inline());
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Try to transform inline content (potentially containing an RPNG) to block content
fn transform_to_block(inline: &InlineContent) -> Option<BlockContent> {
    match inline {
        InlineContent::Link(link) => {
            // Try to get a node from the inner content
            if link.content.len() == 1 {
                if let Some(block) = transform_to_block(&link.content[0]) {
                    return Some(block);
                }
            }
            // Fallback to fetching node from the link's URL
            // TODO
            None
        }
        InlineContent::ImageObject(image) => {
            // Try to get the node from the caption
            if let Some(caption) = image.caption.as_deref() {
                if let Ok(node) = JsonCodec::from_str(caption, None) {
                    if node.is_block() {
                        return Some(node.to_block());
                    }
                }
            }
            // Fallback to getting from the image, either a data URI, or a file
            let url = &image.content_url;
            if url.starts_with("data:image/png;") {
                if let Ok(node) = RpngCodec::from_str(url, None) {
                    if node.is_block() {
                        return Some(node.to_block());
                    }
                }
            }
            let file = PathBuf::from(url);
            if file.exists() {
                if let Ok(node) = executor::block_on(RpngCodec::from_path(&file, None)) {
                    if node.is_block() {
                        return Some(node.to_block());
                    }
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use test_utils::common::tokio;
    use test_utils::{insta::assert_json_snapshot, snapshot_fixtures_content};

    use super::*;

    #[test]
    fn pandoc_fragments() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        snapshot_fixtures_content("fragments/pandoc/*.json", |content| {
            let json =
                runtime.block_on(async { decode_fragment(content, "pandoc", &[]).await.unwrap() });
            assert_json_snapshot!(json);
        });
    }
}
