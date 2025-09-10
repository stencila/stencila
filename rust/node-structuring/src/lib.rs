use clap::ValueEnum;
use strum::{Display, EnumIter, IntoEnumIterator};

use stencila_schema::{Block, CodeInline, Inline, RawBlock, VisitorMut, WalkControl, WalkNode};

use crate::{first::FirstWalk, second::SecondWalk, third::ThirdWalk};

mod first;
mod second;
mod third;

#[cfg(test)]
mod tests;

/// Structuring operation to perform or not
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub enum StructuringOperation {
    /// Extract the title of the work from the first heading
    ///
    /// To be extracted as a title, the heading must have no numbering, be a
    /// level 1 or 2 heading, not be a recognized section type (e.g.
    /// "Abstract"), and can not be after the first primary heading (e.g.
    /// "Introduction")
    HeadingTitle,

    /// Extract keywords from the "Keywords" section, if any
    SectionKeywords,

    /// Extract keywords from any paragraph starting with "Keywords" or similar.
    ParagraphKeywords,

    /// Extract abstract from the "Abstract" section, if any
    SectionAbstract,

    /// Remove any frontmatter
    ///
    /// Content is considered to be frontmatter if it occurs before the first
    /// primary heading (e.g. "Abstract", "Introduction"). In scholarly articles
    /// authors and their affiliations usually occur between the title of the
    /// work and the abstract or introduction. Note that this operation does not
    /// affect the `heading-title`, `heading-keywords`, and `paragraph-keywords`
    /// operations (despite those often occurring before the first primary
    /// section).
    RemoveFrontmatter,

    /// Create a section for each heading
    HeadingSections,

    /// Combine an image with a figure caption before or after it into a figure
    ///
    /// A heading or paragraph is treated as a figure caption if it starts with
    /// "Figure" or "Fig." (case insensitive) followed by a number or
    /// letter-number combination, and the remaining text starts with an
    /// uppercase letter or punctuation. This excludes paragraphs starting like
    /// "Figure 2 shows that..." while including those like "Figure 1. Plot
    /// of results" or "Figure 2: Summary diagram".
    FigureCaptions,

    /// Combine a table caption with the following table or datatable
    ///
    /// A heading or paragraph is treated as a table caption if it starts with
    /// "Table" (case insensitive) followed by a number or letter-number
    /// combination, and the remaining text starts with an uppercase letter or
    /// punctuation. This excludes paragraphs starting like "Table 2 shows
    /// that..." while including those like "Table 1. Summary of results" or
    /// "Table 2: Data analysis".
    TableCaptions,

    /// Transform tables into datatables if possible
    ///
    /// Converts tables to typed datatables when they meet uniformity
    /// requirements: consistent row lengths, simple text-only cells, and no
    /// column/row spans.
    TableDatatable,

    /// Remove empty headings
    ///
    /// A heading is considered empty if it have not context after any numbering
    /// prefix is removed.
    RemoveEmptyHeadings,

    /// Remove empty paragraphs
    ///
    /// A paragraph is considered empty if it contains no content or only
    /// whitespace-only text nodes.
    RemoveEmptyParagraphs,

    /// Remove empty lists
    ///
    /// A list is considered empty if it contains no items or all items are
    /// empty (contain no content or only whitespace).
    RemoveEmptyLists,

    /// Remove empty text nodes
    ///
    /// Text nodes that contain only whitespace characters are removed from
    /// inline content.
    RemoveEmptyText,

    /// Extract structured citations from plain text
    TextCitations,

    /// Extract structured citations from inline math
    ///
    /// Some OCR systems will create inline math for citations, especially for
    /// superscripted numeric citations. This operation will detect numeric
    /// citations in math and create structure citations from them similarly to
    /// `text-citations`.
    MathCitations,

    /// Extract structured links from plain text
    TextLinks,

    /// Extract references from the "References" section, if any
    SectionReferences,
}

/// Citation style options for in-text citations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum CitationStyle {
    /// Author-year citations like (Smith, 2023)
    AuthorYear,

    /// Bracketed numeric citations like [1]
    BracketedNumeric,

    /// Parenthetic numeric citations like (1)
    ParentheticNumeric,

    /// Superscripted numeric citations like ¹
    SuperscriptedNumeric,
}

impl CitationStyle {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::BracketedNumeric | Self::ParentheticNumeric | Self::SuperscriptedNumeric
        )
    }
}

/// Options for document structuring
#[derive(Debug, Clone)]
pub struct StructuringOptions {
    /// Structuring operations to include
    ///
    /// Defaults to all operations
    include_ops: Vec<StructuringOperation>,

    /// Structuring operations to exclude
    ///
    /// Defaults to empty.
    exclude_ops: Vec<StructuringOperation>,

    /// The citation style to assume for text-to-citation structuring.
    ///
    /// If not specified, will be determined automatically based on whether references
    /// are numbered and the relative frequency of detected styles within text.
    /// Only relevant if `text-citations` operation is enabled.
    pub citation_style: Option<CitationStyle>,
}

impl Default for StructuringOptions {
    fn default() -> Self {
        Self {
            include_ops: StructuringOperation::iter().collect(),
            exclude_ops: Vec::new(),
            citation_style: None,
        }
    }
}

impl StructuringOptions {
    /// Whether a structuring operation should be performed
    pub fn should_perform(&self, op: StructuringOperation) -> bool {
        self.include_ops.contains(&op) && !self.exclude_ops.contains(&op)
    }
}

/// Add structure to a document with default options
pub fn structuring<T: WalkNode>(node: &mut T) {
    structuring_with_options(node, StructuringOptions::default())
}

/// Add structure to a document with custom options
pub fn structuring_with_options<T: WalkNode>(node: &mut T, options: StructuringOptions) {
    let mut first = FirstWalk::new(options.clone());
    first.walk(node);
    first.determine_citation_style(options.citation_style);

    let mut second = SecondWalk::new(options.clone(), first);
    second.walk(node);

    let mut third = ThirdWalk::new(options);
    third.walk(node);
}

const REMOVE_MARKER: &str = "<remove>";

/// Create a block that is marked for removal in a subsequent walk
fn block_to_remove(block: &mut Block) -> WalkControl {
    *block = Block::RawBlock(RawBlock::new(REMOVE_MARKER.into(), "".into()));
    WalkControl::Break
}

/// Test if a block should be removed in the current walk
fn should_remove_block(block: &Block) -> bool {
    if let Block::RawBlock(RawBlock { format, .. }) = block {
        format == REMOVE_MARKER
    } else {
        false
    }
}

/// Create an inline that is marked for removal in a subsequent walk
fn inline_to_remove(inline: &mut Inline) -> WalkControl {
    *inline = Inline::CodeInline(CodeInline {
        programming_language: Some(REMOVE_MARKER.into()),
        ..Default::default()
    });
    WalkControl::Break
}

/// Test if an inline should be removed in the current walk
fn should_remove_inline(inline: &Inline) -> bool {
    if let Inline::CodeInline(CodeInline {
        programming_language: Some(lang),
        ..
    }) = inline
    {
        lang == REMOVE_MARKER
    } else {
        false
    }
}
