use std::collections::BTreeMap;

use async_recursion::async_recursion;
use codec::{
    common::{eyre::Result, once_cell::sync::Lazy, regex::Regex, serde_json},
    stencila_schema::{
        Article, BlockContent, CreativeWorkTitle, Delete, Emphasis, Heading, ImageObjectSimple,
        InlineContent, Link, ListItem, Node, NontextualAnnotation, Note, NoteNoteType, Paragraph,
        Strong, TableCell, TableCellContent, TableRow, TableSimple, ThematicBreak,
        ThingIdentifiers,
    },
};

use crate::gdoc;

// See https://github.com/stencila/encoda/blob/master/src/codecs/gdoc/index.ts
// for a previous TypeScript implementation.

// See https://developers.google.com/docs/api/reference/rest/v1/documents#Document
// for browse-able schema for Google Docs.

/**
 * Decode the JSON content of a Google Doc into a Stencila `Node`
 */
pub(crate) async fn decode_async(content: &str) -> Result<Node> {
    let doc: gdoc::Document = serde_json::from_str(content)?;
    let article = document_to_article(doc).await;
    Ok(Node::Article(article))
}

/**
 * Decode the JSON content of a Google Doc into a Stencila `Node` synchronously
 */
pub(crate) fn decode_sync(content: &str) -> Result<Node> {
    futures::executor::block_on(async { decode_async(content).await })
}

/// The decoding context
struct Context {
    /// The map of Google Docs inline objects (e.g. images)
    inline_objects: BTreeMap<String, gdoc::InlineObject>,

    /// The map of Google Docs footnotes
    footnotes: BTreeMap<String, gdoc::Footnote>,

    /// The map of Google Docs lists containing metadata such as bullet styles
    /// and nesting depth
    list_map: BTreeMap<String, gdoc::List>,

    /// The Stencila list items for the current list (needed to be able to collect
    /// paragraphs into lists)
    list_items: BTreeMap<String, Vec<ListItem>>,

    /// The current nesting depth of lists (used to determine if we need to pop
    /// the items into a new list)
    list_depth: u64,
}

/**
 * Transform a Google Doc `Document` to a vector of Stencila `Article`
 */
async fn document_to_article(doc: gdoc::Document) -> Article {
    let identifiers = doc.document_id.map(|id| {
        vec![ThingIdentifiers::String(
            ["https://docs.google.com/document/d/", &id].concat(),
        )]
    });

    let title = doc
        .title
        .map(|string| Box::new(CreativeWorkTitle::String(string)));

    let content = if let Some(body) = doc.body {
        let mut context = Context {
            inline_objects: doc.inline_objects.unwrap_or_default(),
            footnotes: doc.footnotes.unwrap_or_default(),
            list_map: doc.lists.unwrap_or_default(),
            list_items: BTreeMap::new(),
            list_depth: 0,
        };
        Some(body_to_blocks(body, &mut context).await)
    } else {
        None
    };

    Article {
        identifiers,
        title,
        content,
        ..Default::default()
    }
}

/**
 * Transform a Google Doc `Body` to Stencila `BlockContent` nodes
 *
 * Note: the first element in the body is always a section break so we
 * exclude it.
 */
async fn body_to_blocks(body: gdoc::Body, context: &mut Context) -> Vec<BlockContent> {
    let mut blocks: Vec<BlockContent> = Vec::new();
    for elem in body.content.into_iter().flatten() {
        if let Some(block) = structural_element_to_block(elem, context).await {
            blocks.push(block)
        }
    }

    if let Some(BlockContent::ThematicBreak(..)) = blocks.first() {
        blocks[1..].to_vec()
    } else {
        blocks
    }
}

/**
 * Transform a Google Doc `StructuralElement` to Stencila `BlockContent` nodes
 *
 * Note that table of content elements are ignored.
 */
#[async_recursion]
async fn structural_element_to_block(
    elem: gdoc::StructuralElement,
    context: &mut Context,
) -> Option<BlockContent> {
    if let Some(paragraph) = elem.paragraph {
        paragraph_to_block(paragraph, context).await
    } else if let Some(..) = elem.section_break {
        section_break_to_block()
    } else if let Some(table) = elem.table {
        table_to_block(table, context).await
    } else if let Some(..) = elem.table_of_contents {
        // Ignore table of contents
        None
    } else {
        unreachable!("A `StructuralElement` should have one of the above properties")
    }
}

/**
 * Transform a Google Doc `Paragraph` to one or more Stencila `BlockContent` nodes
 *
 * Usually, the paragraph will be decoded to a `Paragraph`, `Heading` or `List`.
 * However, if the paragraph contains only one element and that element
 * is a reproducible image, then it will be decoded to the entity in that image
 * e.g. `CodeChunk`.
 */
async fn paragraph_to_block(para: gdoc::Paragraph, context: &mut Context) -> Option<BlockContent> {
    let mut inlines = Vec::new();
    for elem in para.elements.into_iter().flatten() {
        if let Some(inline) = paragraph_element_to_inline(elem, context).await {
            inlines.push(inline)
        }
    }

    if let Some(style) = para.paragraph_style {
        if let Some(style_name) = style.named_style_type {
            static REGEX: Lazy<Regex> =
                Lazy::new(|| Regex::new("^HEADING_([1-9])$").expect("Unable to create regex"));

            if let Some(captures) = REGEX.captures(&style_name) {
                let depth = captures
                    .get(1)
                    .and_then(|group| group.as_str().parse().ok())
                    .or(Some(1));

                return Some(BlockContent::Heading(Heading {
                    content: inlines,
                    depth,
                    ..Default::default()
                }));
            }
        }
    }

    if let Some(bullet) = para.bullet {
        if let Some(_list_id) = bullet.list_id {
            let _list_level = bullet.nesting_level.unwrap_or(0);
        }
    }

    Some(BlockContent::Paragraph(Paragraph {
        content: inlines,
        ..Default::default()
    }))
}

/**
 * Transform a Google Doc `SectionBreak` to a Stencila `ThematicBreak`
 */
fn section_break_to_block() -> Option<BlockContent> {
    Some(BlockContent::ThematicBreak(ThematicBreak::default()))
}

/**
 * Transform a Google Doc `Table` to a Stencila `Table`
 */
async fn table_to_block(table: gdoc::Table, context: &mut Context) -> Option<BlockContent> {
    let mut rows: Vec<TableRow> = Vec::new();
    for row in table.table_rows.into_iter().flatten() {
        let row = table_row_to_table_row(row, context).await;
        rows.push(row)
    }

    Some(BlockContent::Table(TableSimple {
        rows,
        ..Default::default()
    }))
}

/**
 * Transform a Google Doc `TableRow` to a Stencila `TableRow`
 */
async fn table_row_to_table_row(table_row: gdoc::TableRow, context: &mut Context) -> TableRow {
    let mut cells: Vec<TableCell> = Vec::new();
    for cell in table_row.table_cells.into_iter().flatten() {
        let cell = table_cell_to_table_cell(cell, context).await;
        cells.push(cell)
    }

    TableRow {
        cells,
        ..Default::default()
    }
}

/**
 * Transform a Google Doc `TableCell` to a Stencila `TableCell`
 */
async fn table_cell_to_table_cell(table_cell: gdoc::TableCell, context: &mut Context) -> TableCell {
    let mut blocks: Vec<BlockContent> = Vec::new();
    for elem in table_cell.content.into_iter().flatten() {
        if let Some(block) = structural_element_to_block(elem, context).await {
            blocks.push(block)
        }
    }

    let content = if blocks.is_empty() {
        None
    } else if let BlockContent::Paragraph(Paragraph { content, .. }) = &blocks[0] {
        if content.len() == 1 {
            Some(TableCellContent::VecInlineContent(content.clone()))
        } else {
            Some(TableCellContent::VecBlockContent(blocks))
        }
    } else {
        Some(TableCellContent::VecBlockContent(blocks))
    };

    TableCell {
        content,
        ..Default::default()
    }
}

/**
 * Transform a Google Doc `ParagraphElement` to a Stencila `InlineContent` node
 */
#[allow(clippy::if_same_then_else)]
async fn paragraph_element_to_inline(
    elem: gdoc::ParagraphElement,
    context: &mut Context,
) -> Option<InlineContent> {
    if let Some(text_run) = elem.text_run {
        text_run_to_inline(text_run)
    } else if let Some(inline_object_element) = elem.inline_object_element {
        inline_object_element_to_inline(inline_object_element, context).await
    } else if let Some(footnote_reference) = elem.footnote_reference {
        footnote_reference_to_inline(footnote_reference, context).await
    } else if let Some(person) = elem.person {
        person_to_inline(person)
    } else if let Some(rich_link) = elem.rich_link {
        rick_link_to_inline(rich_link)
    } else if matches!(elem.page_break, Some(..))
        || matches!(elem.column_break, Some(..))
        || matches!(elem.horizontal_rule, Some(..))
    {
        // Explicitly ignore these non-semantic elements
        None
    } else if matches!(elem.auto_text, Some(..)) || matches!(elem.equation, Some(..)) {
        // Explicitly ignore these elements that do not have content (?)
        None
    } else {
        unreachable!("A `ParagraphElement` should have one of the above properties")
    }
}

/**
 * Transform a Google Doc `TextRun` to a `string`, `Emphasis`, `Strong`, `Delete`,
 * `Link`, `Subscript` or `Superscript` node.
 *
 * A `TextRun` can have multiple styles and this function nests them in
 * a the order they are listed at https://developers.google.com/docs/api/reference/rest/v1/documents#TextStyle
 * (i.e. with `Link` as the outer node)
 */
fn text_run_to_inline(text_run: gdoc::TextRun) -> Option<InlineContent> {
    let mut string = text_run.content.unwrap_or_default();
    if string.ends_with('\n') {
        string.pop();
    };
    if string.is_empty() {
        return None;
    }

    let mut inline = InlineContent::String(string);

    if let Some(text_style) = text_run.text_style {
        if let Some(true) = text_style.bold {
            inline = InlineContent::Strong(Strong {
                content: vec![inline],
                ..Default::default()
            });
        }

        if let Some(true) = text_style.italic {
            inline = InlineContent::Emphasis(Emphasis {
                content: vec![inline],
                ..Default::default()
            });
        }

        if let Some(true) = text_style.underline {
            inline = InlineContent::NontextualAnnotation(NontextualAnnotation {
                content: vec![inline],
                ..Default::default()
            });
        }

        if let Some(true) = text_style.strikethrough {
            inline = InlineContent::Delete(Delete {
                content: vec![inline],
                ..Default::default()
            });
        }

        if let Some(link) = text_style.link {
            // A `Link` has one of the following
            // https://developers.google.com/docs/api/reference/rest/v1/documents#Link
            let target = link
                .url
                .or_else(|| link.bookmark_id.map(|id| ["#", &id].concat()))
                .or_else(|| link.heading_id.map(|id| ["#", &id].concat()))
                .unwrap_or_default();
            inline = InlineContent::Link(Link {
                content: vec![inline],
                target,
                ..Default::default()
            });
        }
    }

    Some(inline)
}

/**
 * Transform a Google Doc `InlineObjectElement` to a Stencila `ImageObjectSimple`.
 */
async fn inline_object_element_to_inline(
    inline_object_element: gdoc::InlineObjectElement,
    context: &mut Context,
) -> Option<InlineContent> {
    let embedded_object = inline_object_element
        .inline_object_id
        .and_then(|id| context.inline_objects.get(&id).cloned())
        .and_then(|inline_object| inline_object.inline_object_properties)
        .and_then(|inline_object_props| inline_object_props.embedded_object);

    embedded_object.and_then(|embedded_object| {
        let title = embedded_object
            .title
            .map(|title| Box::new(CreativeWorkTitle::String(title)));
        let _caption = embedded_object.description;

        embedded_object.image_properties.map(|image_properties| {
            InlineContent::ImageObject(ImageObjectSimple {
                title,
                //caption,
                content_url: image_properties.content_uri.unwrap_or_default(),
                ..Default::default()
            })
        })
    })
}

/**
 * Transform a Google Doc `FootnoteReference` to a Stencila `Note`.
 */
async fn footnote_reference_to_inline(
    footnote_reference: gdoc::FootnoteReference,
    context: &mut Context,
) -> Option<InlineContent> {
    if let Some(footnote) = footnote_reference
        .footnote_id
        .and_then(|id| context.footnotes.remove(&id))
    {
        let mut content: Vec<BlockContent> = Vec::new();
        for elem in footnote.content.into_iter().flatten() {
            if let Some(block) = structural_element_to_block(elem, context).await {
                content.push(block)
            }
        }
        Some(InlineContent::Note(Note {
            note_type: Some(NoteNoteType::Footnote),
            content,
            ..Default::default()
        }))
    } else {
        None
    }
}

/**
 * Transform a Google Doc `Person` to a Stencila `String`.
 */
fn person_to_inline(person: gdoc::Person) -> Option<InlineContent> {
    person.person_properties.map(|props| {
        let mut repr = String::new();
        if let Some(name) = props.name {
            repr = name;
        }
        if let Some(email) = props.email {
            if !repr.is_empty() {
                repr.push(' ');
            }
            repr.push_str(&email);
        }
        InlineContent::String(repr)
    })
}

/**
 * Transform a Google Doc `RichLink` to a Stencila `Link`.
 *
 * According to https://developers.google.com/docs/api/reference/rest/v1/documents#RichLinkProperties
 * `uri` and `target` are always present.
 */
fn rick_link_to_inline(rich_link: gdoc::RichLink) -> Option<InlineContent> {
    rich_link.rich_link_properties.map(|props| {
        let target = props.uri.unwrap_or_default();
        let title = props.title.unwrap_or_else(|| "untitled".to_string());
        InlineContent::Link(Link {
            target,
            title: Some(Box::new(title.clone())),
            content: vec![InlineContent::String(title)],
            ..Default::default()
        })
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures_content};

    #[test]
    fn decode_gdoc_articles() {
        snapshot_fixtures_content("articles/gdoc/*.gdoc", |content| {
            assert_json_snapshot!(decode_sync(content).unwrap());
        });
    }
}
