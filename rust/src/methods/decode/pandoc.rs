use crate::{
    binaries::{self, BinaryInstallation},
    methods::coerce::coerce,
};
use defaults::Defaults;
use eyre::{bail, Result};
use pandoc_types::definition as pandoc;
use std::{collections::HashMap, io::Write, path::PathBuf, process::Stdio};
use stencila_schema::{
    Article, BlockContent, Cite, CiteCitationMode, CiteGroup, CodeBlock, CodeFragment, Delete,
    Emphasis, Heading, ImageObjectSimple, InlineContent, Link, List, ListItem, ListItemContent,
    ListOrder, MathFragment, Node, NontextualAnnotation, Paragraph, Quote, QuoteBlock, Strong,
    Subscript, Superscript, TableCaption, TableCell, TableCellContent, TableRow, TableRowRowType,
    TableSimple, ThematicBreak,
};
use tokio::sync::OnceCell;

static PANDOC: OnceCell<BinaryInstallation> = OnceCell::const_new();

/// Decoding options for the `decode` and `decode_fragment` functions
#[derive(Clone, Defaults)]
pub struct Options {
    /// The format of the input
    #[def = "\"pandoc\".to_string()"]
    pub format: String,

    /// Whether the content is a file system path or not
    #[def = "false"]
    pub is_file: bool,
}

/// Decode a document to a `Node`
///
/// Intended for decoding an entire document into an `Article`
/// (and, in the future, potentially other types).
pub async fn decode(input: &str, options: Options) -> Result<Node> {
    let pandoc = decode_pandoc(input, &options).await?;

    let context = Context {
        options: options.clone(),
    };

    let mut article = translate_meta(pandoc.0, &context)?;

    let content = translate_blocks(&pandoc.1, &context);
    article.content = if content.is_empty() {
        None
    } else {
        Some(content)
    };

    Ok(Node::Article(article))
}

/// Decode a fragment to a vector of `BlockContent`
///
/// Intended for decoding a fragment of a larger document (e.g. from some Latex in
/// a Markdown document). Ignores any meta data e.g. title
pub async fn decode_fragment(input: &str, options: Options) -> Result<Vec<BlockContent>> {
    if input.is_empty() {
        return Ok(vec![]);
    }

    let pandoc = decode_pandoc(input, &options).await?;

    let context = Context {
        options: options.clone(),
    };

    Ok(translate_blocks(&pandoc.1, &context))
}

/// Decode some content (either a string or file path) to a Pandoc document
///
/// Calls Pandoc binary to convert the content to Pandoc JSON which is then deserialized to
/// a Pandoc element tree which is translated to a Stencila node tree.
///
/// The version of Pandoc required is partially based on compatibility with the `pandoc_types`
/// crate. Some recent changes to pandoc to Pandoc types used by Pandoc (from https://pandoc.org/releases.html):
///
///   pandoc 2.11 (2020-10-11) : pandoc-types 1.22
///   pandoc 2.10 (2020-06-29) : pandoc-types 1.21
async fn decode_pandoc(input: &str, options: &Options) -> Result<pandoc::Pandoc> {
    // Get the Pandoc JSON
    let json = if options.format == "pandoc" {
        input.to_string()
    } else {
        let binary = PANDOC
            .get_or_try_init(|| binaries::require("pandoc", "2.11"))
            .await?;

        let mut command = binary.command();
        command
            .args(&[
                format!("--from={}", options.format),
                "--to=json".to_string(),
            ])
            .stdout(Stdio::piped());

        let child = if options.is_file {
            if !PathBuf::from(input).exists() {
                bail!("File does not exists: {}", input)
            }
            command.arg(input).spawn()?
        } else {
            let mut child = command.stdin(Stdio::piped()).spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input.as_ref())?;
            }
            child
        };

        let output = child.wait_with_output()?;

        std::str::from_utf8(output.stdout.as_ref())?.to_string()
    };

    Ok(serde_json::from_str(&json)?)
}

/// Translate Pandoc meta data into an `Article` node
fn translate_meta(meta: pandoc::Meta, context: &Context) -> Result<Article> {
    let mut article = translate_meta_map(&meta.0, context);
    article.insert("type".to_string(), serde_json::json!("Article"));

    let node = coerce(serde_json::Value::Object(article))?;

    match node {
        Node::Article(article) => Ok(article),
        _ => bail!("Expected an article"),
    }
}

/// Translate a map of `MetaValue` to a map of `serde_json::Value`
fn translate_meta_map(
    map: &HashMap<String, pandoc::MetaValue>,
    context: &Context,
) -> serde_json::Map<String, serde_json::Value> {
    map.iter()
        .map(|(key, value)| (key.clone(), translate_meta_value(value, context)))
        .collect()
}

/// Translate a meta value to a `serde_json::Value`
fn translate_meta_value(value: &pandoc::MetaValue, context: &Context) -> serde_json::Value {
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
            translate_inlines(&inlines, context)
                .iter()
                .map(|inline| serde_json::to_value(inline).expect("Can serialize to JSON value"))
                .collect(),
        ),
        pandoc::MetaValue::MetaBlocks(blocks) => serde_json::Value::Array(
            translate_blocks(&blocks, context)
                .iter()
                .map(|block| serde_json::to_value(block).expect("Can serialize to JSON value"))
                .collect(),
        ),
    }
}

/// Decoding context
struct Context {
    #[allow(dead_code)]
    options: Options,
}

/// Translate a vector of Pandoc `Block` elements to a vector of `BlockContent` nodes
fn translate_blocks(elements: &[pandoc::Block], context: &Context) -> Vec<BlockContent> {
    elements
        .iter()
        .flat_map(|child| translate_block(&child, context))
        .collect()
}

/// Translate a Pandoc `Block` element into a zero or more `BlockContent` nodes
///
/// Will ignore elements that are dealt with by `translate_inline`
fn translate_block(element: &pandoc::Block, context: &Context) -> Vec<BlockContent> {
    match element {
        pandoc::Block::Header(depth, attrs, inlines) => {
            let id = get_id(attrs);
            let depth = Some(Box::new(*depth as i64));
            vec![BlockContent::Heading(Heading {
                id,
                depth,
                content: translate_inlines(inlines, context),
                ..Default::default()
            })]
        }

        pandoc::Block::Para(inlines) => vec![BlockContent::Paragraph(Paragraph {
            content: translate_inlines(inlines, context),
            ..Default::default()
        })],

        pandoc::Block::BlockQuote(blocks) => {
            vec![BlockContent::QuoteBlock(QuoteBlock {
                content: translate_blocks(blocks, context),
                ..Default::default()
            })]
        }

        pandoc::Block::CodeBlock(attrs, text) => {
            let id = get_id(attrs);
            let programming_language = get_attr(attrs, "classes").map(|value| Box::new(value));
            vec![BlockContent::CodeBlock(CodeBlock {
                id,
                programming_language,
                text: text.clone(),
                ..Default::default()
            })]
        }

        pandoc::Block::OrderedList(.., items) | pandoc::Block::BulletList(items) => {
            let order = Some(Box::new(match element {
                pandoc::Block::OrderedList(..) => ListOrder::Ascending,
                _ => ListOrder::Unordered,
            }));
            let items = items
                .iter()
                .map(|blocks| {
                    let blocks = translate_blocks(blocks, context);
                    let content = if blocks.is_empty() {
                        None
                    } else {
                        Some(Box::new(ListItemContent::VecBlockContent(blocks)))
                    };
                    ListItem {
                        content,
                        ..Default::default()
                    }
                })
                .collect();
            vec![BlockContent::List(List {
                order,
                items,
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
                        Some(Box::new(ListItemContent::VecBlockContent(blocks)))
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
                .map(|row| translate_row(row, context, Some(Box::new(TableRowRowType::Header))))
                .collect();
            let body: Vec<TableRow> = bodies
                .iter()
                .flat_map(|body| {
                    let intermediate_head: Vec<TableRow> = body
                        .2
                        .iter()
                        .map(|row| {
                            translate_row(row, context, Some(Box::new(TableRowRowType::Header)))
                        })
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
                .map(|row| translate_row(row, context, Some(Box::new(TableRowRowType::Footer))))
                .collect();
            let rows = [head, body, foot].concat();

            vec![BlockContent::Table(TableSimple {
                id,
                caption,
                rows,
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
    context: &Context,
    row_type: Option<Box<TableRowRowType>>,
) -> TableRow {
    let cells = row
        .1
        .iter()
        .map(|cell| translate_cell(cell, context))
        .collect();
    TableRow {
        row_type,
        cells,
        ..Default::default()
    }
}

/// Translate a Pandoc table cell into a `TableCell`
///
/// Pandoc always returns blocks for table cells, for example, even for
/// a single number it returns a paragraph. This is somewhat heavy weight,
/// and inconsistent with other decoders in this repo, so here we unwrap
/// a single paragraph to inlines.
fn translate_cell(cell: &pandoc::Cell, context: &Context) -> TableCell {
    let pandoc::Cell(_attrs, _alignment, row_span, col_span, blocks) = cell;
    let blocks = translate_blocks(blocks, context);
    let content = match blocks.len() {
        0 => None,
        1 => match &blocks[0] {
            BlockContent::Paragraph(paragraph) => Some(Box::new(
                TableCellContent::VecInlineContent(paragraph.content.clone()),
            )),
            _ => Some(Box::new(TableCellContent::VecBlockContent(blocks))),
        },
        _ => Some(Box::new(TableCellContent::VecBlockContent(blocks))),
    };
    let rowspan = match row_span {
        1 => None,
        _ => Some(Box::new((*row_span) as i64)),
    };
    let colspan = match col_span {
        1 => None,
        _ => Some(Box::new((*col_span) as i64)),
    };
    TableCell {
        content,
        rowspan,
        colspan,
        ..Default::default()
    }
}

/// Translate a vector of Pandoc `Inline` elements to a vector of `InlineContent` nodes
fn translate_inlines(elements: &[pandoc::Inline], context: &Context) -> Vec<InlineContent> {
    let mut inlines: Vec<InlineContent> = elements
        .iter()
        .flat_map(|child| translate_inline(&child, context))
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
fn translate_inline(element: &pandoc::Inline, context: &Context) -> Vec<InlineContent> {
    match element {
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
        pandoc::Inline::Image(attrs, _inlines, target) => {
            // TODO: Return audio or video depending upon the file extension
            // TODO: Translate inlines as content. Perhaps wait for possible change
            // of `ImageObject.content` to `Vec<InlineContent>`.
            let pandoc::Target(url, title) = target;
            vec![InlineContent::ImageObject(ImageObjectSimple {
                id: get_id(attrs),
                caption: get_string_prop(title),
                content_url: url.clone(),
                ..Default::default()
            })]
        }

        pandoc::Inline::Cite(citations, _inlines) => {
            let items: Vec<Cite> = citations
                .iter()
                .map(|citation| {
                    // TODO: Use inlines and prefix and suffix (consider using Vec<InlineContent> for both
                    // and parsing out pagination)
                    let citation_mode = Some(Box::new(match citation.citation_mode {
                        pandoc::CitationMode::NormalCitation => CiteCitationMode::Parenthetical,
                        pandoc::CitationMode::AuthorInText => CiteCitationMode::Narrative,
                        pandoc::CitationMode::SuppressAuthor => CiteCitationMode::NarrativeYear,
                    }));
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
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[test]
    fn pandoc_fragments() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        snapshot_content("fragments/pandoc/*.json", |content| {
            let json = runtime.block_on(async {
                decode_fragment(
                    &content,
                    Options {
                        format: "pandoc".to_string(),
                        is_file: false,
                    },
                )
                .await
                .unwrap()
            });
            assert_json_snapshot!(json);
        });
    }
}
