use crate::from_pandoc;
use codec_json::JsonCodec;
use codec_rpng::RpngCodec;
use codec_trait::{
    eyre::{bail, Result},
    stencila_schema::*,
    Codec,
};
use codec_txt::ToTxt;
use formats::{FormatNodeType, FORMATS};
use node_coerce::coerce;
use pandoc_types::definition as pandoc;
use std::{collections::HashMap, path::PathBuf};

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
    Ok(translate_blocks(&pandoc.1, &context))
}

/// Decode a Pandoc document to a `Node`
pub fn decode_pandoc(pandoc: pandoc::Pandoc) -> Result<Node> {
    let context = DecodeContext {};
    let mut article = translate_meta(pandoc.0, &context)?;

    let content = translate_blocks(&pandoc.1, &context);
    article.content = if content.is_empty() {
        None
    } else {
        Some(content)
    };

    Ok(Node::Article(article))
}

/// Translate Pandoc meta data into an `Article` node
fn translate_meta(meta: pandoc::Meta, context: &DecodeContext) -> Result<Article> {
    let mut article = translate_meta_map(&meta.0, context);
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
            vec![BlockContent::Heading(Heading {
                id: get_id(attrs),
                depth: Some(*depth as u8),
                content: translate_inlines(inlines, context),
                ..Default::default()
            })]
        }

        pandoc::Block::Para(inlines) => {
            let content = translate_inlines(inlines, context);
            if content.len() == 1 {
                if let Some(translated) = try_code_chunk(&content[0]) {
                    return vec![translated];
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

        pandoc::Block::CodeBlock(attrs, text) => {
            let id = get_id(attrs);
            let programming_language = get_attr(attrs, "classes").map(Box::new);
            vec![BlockContent::CodeBlock(CodeBlock {
                id,
                programming_language,
                text: text.clone(),
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

        pandoc::Block::Table(attrs, caption, _column_specs, head, bodies, foot) => {
            let id = get_id(attrs);

            let caption = translate_blocks(&caption.1, context);
            let caption = match caption.is_empty() {
                true => None,
                false => Some(Box::new(TableCaption::VecBlockContent(caption))),
            };

            let head: Vec<TableRow> = head
                .1
                .iter()
                .map(|row| translate_row(row, context, Some(TableRowRowType::Header)))
                .collect();
            let body: Vec<TableRow> = bodies
                .iter()
                .flat_map(|body| {
                    let intermediate_head: Vec<TableRow> = body
                        .2
                        .iter()
                        .map(|row| translate_row(row, context, Some(TableRowRowType::Header)))
                        .collect();
                    let intermediate_body: Vec<TableRow> = body
                        .3
                        .iter()
                        .map(|row| translate_row(row, context, None))
                        .collect();
                    [intermediate_head, intermediate_body].concat()
                })
                .collect();
            let foot: Vec<TableRow> = foot
                .1
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
    let cells = row
        .1
        .iter()
        .map(|cell| translate_cell(cell, context))
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
fn translate_cell(cell: &pandoc::Cell, context: &DecodeContext) -> TableCell {
    let pandoc::Cell(_attrs, _alignment, row_span, col_span, blocks) = cell;
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
            vec![InlineContent::NontextualAnnotation(NontextualAnnotation {
                content: translate_inlines(inlines, context),
                ..Default::default()
            })]
        }
        pandoc::Inline::Strong(inlines) => vec![InlineContent::Strong(Strong {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],
        pandoc::Inline::Strikeout(inlines) => vec![InlineContent::Delete(Delete {
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

        pandoc::Inline::Code(attrs, text) => {
            let id = get_id(attrs);
            vec![InlineContent::CodeFragment(CodeFragment {
                id,
                text: text.clone(),
                ..Default::default()
            })]
        }
        pandoc::Inline::Math(_math_type, text) => vec![InlineContent::MathFragment(MathFragment {
            text: text.clone(),
            ..Default::default()
        })],

        pandoc::Inline::Link(attrs, inlines, target) => {
            let pandoc::Target(url, title) = target;
            vec![InlineContent::Link(Link {
                id: get_id(attrs),
                target: url.clone(),
                title: get_string_prop(title),
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

            let pandoc::Target(url, title) = target;
            let content_url = url.clone();
            let title = match title.is_empty() {
                true => None,
                false => Some(Box::new(CreativeWorkTitle::String(title.to_string()))),
            };

            match FORMATS.match_path(&content_url).node_type {
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
        .map(|inline| {
            if let Some(code_expression) = try_code_expression(&inline) {
                code_expression
            } else {
                inline
            }
        })
        .collect()
}

/// Get an attribute from a Pandoc `Attr` tuple struct
fn get_attr(attrs: &pandoc::Attr, name: &str) -> Option<String> {
    match name {
        "id" => match attrs.0.is_empty() {
            true => None,
            false => Some(attrs.0.clone()),
        },
        "classes" => match attrs.1.is_empty() {
            true => None,
            false => Some(attrs.1.join(" ")),
        },
        _ => attrs.2.iter().find_map(|(key, value)| {
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
fn get_string_prop(value: &str) -> Option<Box<String>> {
    match value.is_empty() {
        true => None,
        false => Some(Box::new(value.to_string())),
    }
}

// Get the `id` property of a `Entity` from a  Pandoc `Attr` tuple struct
fn get_id(attrs: &pandoc::Attr) -> Option<Box<String>> {
    get_attr(attrs, "id").and_then(|value| get_string_prop(&value))
}

/// Try to extract a `CodeExpression` from an RPNG representation
fn try_code_expression(inline: &InlineContent) -> Option<InlineContent> {
    match inline {
        InlineContent::Link(link) => {
            if link.content.len() == 1 {
                // If this is a link around a code expression and the it has a
                // matching title then return that code expression
                let title = match link.title.as_deref() {
                    Some(title) => title.clone(),
                    None => "".to_string(),
                };
                if title == "CodeExpression"
                    || link.target.starts_with("https://hub.stenci.la/api/nodes")
                {
                    if let InlineContent::CodeExpression(expr) = &link.content[0] {
                        return Some(InlineContent::CodeExpression(expr.clone()));
                    }
                }
                // Try to get a code expression from the inner content
                if let Some(expr) = try_code_expression(&link.content[0]) {
                    return Some(expr);
                }
            }
            // Fallback to fetching code expression from the link's URL
            // TODO
        }
        InlineContent::ImageObject(image) => {
            // Try to get the code expression from the caption
            if let Some(caption) = image.caption.as_deref() {
                if let Ok(Node::CodeExpression(expr)) = JsonCodec::from_str(caption, None) {
                    return Some(InlineContent::CodeExpression(expr));
                }
            }
            // Fallback to getting from the image
            if let Ok(Node::CodeExpression(expr)) = RpngCodec::from_str(&image.content_url, None) {
                return Some(InlineContent::CodeExpression(expr));
            }
        }
        _ => (),
    };
    None
}

/// Try to extract a `CodeChunk` from an RPNG representation
fn try_code_chunk(inline: &InlineContent) -> Option<BlockContent> {
    match inline {
        InlineContent::Link(link) => {
            // Try to get a code chunk from the inner content
            if link.content.len() == 1 {
                if let Some(chunk) = try_code_chunk(&link.content[0]) {
                    return Some(chunk);
                }
            }
            // Fallback to fetching code chunk from the link's URL
            // TODO
        }
        InlineContent::ImageObject(image) => {
            // Try to get the code chunk from the caption
            if let Some(caption) = image.caption.as_deref() {
                if let Ok(Node::CodeChunk(chunk)) = JsonCodec::from_str(caption, None) {
                    return Some(BlockContent::CodeChunk(chunk));
                }
            }
            // Fallback to getting from the image
            if let Ok(Node::CodeChunk(chunk)) = RpngCodec::from_str(&image.content_url, None) {
                return Some(BlockContent::CodeChunk(chunk));
            }
        }
        _ => (),
    };
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures_content};

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
