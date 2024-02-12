///! Functions used in type definitions for specifying property-based generation strategies for node types
use common::itertools::interleave;
pub use proptest_derive::Arbitrary;

pub use proptest::{
    collection::{size_range, vec},
    option,
    prelude::*,
};

use crate::*;

pub use shortcuts::*;

prop_compose! {
    /// Generate a vector of inline content of arbitrary length and content
    /// but always having strings interspersed by other inline content (to separate them
    /// so that they do not get decoded as a single string).
    pub fn vec_inlines(max_size: usize)(
        length in 1..=max_size
    )(
        texts in vec(
            Text::arbitrary().prop_map(Inline::Text),
            size_range(length)
        ),
        others in vec(
            Inline::arbitrary().prop_filter(
                "Interleave text with other inlines",
                |inline| !matches!(inline, Inline::Text(..))
            ),
            size_range(length - 1)
        )
    ) -> Vec<Inline> {
        interleave_inlines(texts, others)
    }
}

prop_compose! {
    /// Generate a vector of inline content of arbitrary length and not containing
    /// any recursive inline types (inlines that contain other inlines)
    pub fn vec_inlines_non_recursive(max_size: usize)(
        length in 1..=max_size
    )(
        texts in vec(
            Text::arbitrary().prop_map(Inline::Text),
            size_range(length)
        ),
        others in vec(
            prop_oneof![
                CodeExpression::arbitrary().prop_map(Inline::CodeExpression),
                CodeInline::arbitrary().prop_map(Inline::CodeInline),
                MathInline::arbitrary().prop_map(Inline::MathInline)
            ],
            size_range(length - 1)
        )
    ) -> Vec<Inline> {
        interleave_inlines(texts, others)
    }
}

/// Interleave inline content
///
/// Restrictions:
///   - Always starts and ends with a string.
///   - Ensures that nodes such as `Strong`, `Emphasis`, and `Strikeout`
///     are surrounded by spaces (for compatibility with Markdown decoding).
///   - No leading or trailing whitespace (for Markdown).
fn interleave_inlines(texts: Vec<Inline>, others: Vec<Inline>) -> Vec<Inline> {
    let mut content: Vec<Inline> = interleave(texts, others).collect();
    for index in 0..content.len() {
        let spaces = matches!(
            content[index],
            Inline::Emphasis(..)
                | Inline::MathInline(..)
                | Inline::Strikeout(..)
                | Inline::Strong(..)
                | Inline::Subscript(..)
                | Inline::Superscript(..)
        );

        if spaces {
            if let Inline::Text(text) = &mut content[index - 1] {
                text.value.push(' ');
            }
            if let Inline::Text(text) = &mut content[index + 1] {
                text.value.insert(0, ' ');
            }
        }

        if index == 0 {
            if let Inline::Text(text) = &mut content[index] {
                if text.value.starts_with(char::is_whitespace) {
                    text.value.insert(0, 'A')
                }
            }
        }

        if index == content.len() - 1 {
            if let Inline::Text(text) = &mut content[index] {
                if text.value.ends_with(char::is_whitespace) {
                    text.value.push('.')
                }
            }
        }
    }
    content
}

prop_compose! {
    /// Generate a vector of block content of arbitrary length and content
    ///
    /// Restrictions:
    ///  - List of same ordering can not be adjacent to each other (in Markdown they
    ///    get decoded as the same list)
    pub fn vec_blocks(max_size: usize)(
        length in 1..=max_size
    )(
        blocks in vec(Block::arbitrary(), size_range(length))
            .prop_filter(
                "Lists with same ordering should not be adjacent",
                |blocks| {
                    for index in 1..blocks.len() {
                        if let (Block::List(prev), Block::List(curr)) = (&blocks[index-1], &blocks[index]) {
                            match (&prev.order, &curr.order) {
                                (ListOrder::Ascending, ListOrder::Ascending) |
                                (ListOrder::Unordered, ListOrder::Unordered) => {
                                    return false
                                },
                                _ => ()
                            }
                        }
                    }
                    true
                }
            )
    ) -> Vec<Block> {
        blocks
    }
}

prop_compose! {
    /// Generate a vector of block content of arbitrary length and not containing
    /// any recursive block types (blocks that contain other blocks)
    pub fn vec_blocks_non_recursive(max_size: usize)(
        length in 1..=max_size
    )(
        blocks in vec(
            prop_oneof![
                CodeBlock::arbitrary().prop_map(Block::CodeBlock),
                CodeChunk::arbitrary().prop_map(Block::CodeChunk),
                Heading::arbitrary().prop_map(Block::Heading),
                MathBlock::arbitrary().prop_map(Block::MathBlock),
                Paragraph::arbitrary().prop_map(Block::Paragraph),
                QuoteBlock::arbitrary().prop_map(Block::QuoteBlock),
                ThematicBreak::arbitrary().prop_map(Block::ThematicBreak),
            ],
            size_range(length)
        )
    ) -> Vec<Block> {
        blocks
    }
}

prop_compose! {
    /// Generate a vector of block content of arbitrary length, only containing
    /// block types expected in lists (and not other lists), and starting with a
    /// paragraph
    pub fn vec_blocks_list_item(max_size: usize)(
        length in 1..=max_size
    )(
        first in Paragraph::arbitrary().prop_map(Block::Paragraph),
        mut rest in vec(
            prop_oneof![
                CodeBlock::arbitrary().prop_map(Block::CodeBlock),
                Paragraph::arbitrary().prop_map(Block::Paragraph),
                QuoteBlock::arbitrary().prop_map(Block::QuoteBlock),
            ],
            size_range(length-1)
        )
    ) -> Vec<Block> {
        let mut blocks = vec![first];
        blocks.append(&mut rest);
        blocks
    }
}

prop_compose! {
    /// Generate a vector of arbitrary paragraphs
    pub fn vec_paragraphs(max_size: usize)(
        length in 1..=max_size
    )(
        blocks in vec(
            Paragraph::arbitrary().prop_map(Block::Paragraph),
            size_range(length)
        )
    ) -> Vec<Block> {
        blocks
    }
}

prop_compose! {
    /// Generate a vector with an arbitrary heading and an arbitrary paragraph
    pub fn vec_heading_paragraph()(
        heading in Heading::arbitrary(),
        paragraph in Paragraph::arbitrary()
    ) -> Vec<Block> {
        vec![Block::Heading(heading), Block::Paragraph(paragraph)]
    }
}

prop_compose! {
    /// Generate a vector of arbitrary table cells of given width
    pub fn table_cells(width: usize)(
        cells in vec(TableCell::arbitrary(), width..=width)
    ) -> Vec<TableCell> {
        cells
    }
}

prop_compose! {
    /// Generate a vector of arbitrary table rows with a first header row.
    pub fn table_rows_with_header(max_width: usize, max_length: usize)(
        width in 1..=max_width,
        length in 1..=max_length
    )(
        header in table_cells(width),
        rest in vec(table_cells(width), size_range(length))
    ) -> Vec<TableRow> {
        let mut rows = vec![TableRow{
            row_type: Some(TableRowType::HeaderRow),
            cells: header,
            ..Default::default()
        }];

        let mut rest: Vec<TableRow> = rest.into_iter().map(|cells| TableRow{
            cells,
            ..Default::default()
        }).collect();
        rows.append(&mut rest);

        rows
    }
}
