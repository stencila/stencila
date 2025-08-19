use std::collections::HashMap;

use codec_biblio::decode::{
    bracketed_numeric_citation, parenthetic_numeric_citation, superscripted_numeric_citation,
    text_to_reference, text_with_author_year_citations, text_with_bracketed_numeric_citations,
    text_with_parenthetic_numeric_citations,
};
use codec_text_trait::to_text;
use common::{once_cell::sync::Lazy, regex::Regex};
use schema::{
    Admonition, Article, Block, Figure, ForBlock, Heading, IncludeBlock, Inline, List, ListOrder,
    MathInline, Node, NodeId, Paragraph, Reference, Section, StyledBlock, Text, VisitorMut,
    WalkControl,
};

/// A type of potential block replacement
#[derive(Debug)]
pub(super) enum BlockReplacement {
    /// Replace an image followed by a caption with a [`Figure`]
    ImageThenCaption,

    /// Replace a caption followed by an image with a [`Figure`]
    CaptionThenImage,

    /// Apply a caption followed by a [`Table`] to the table
    CaptionThenTable,
}

/// A type of potential inline replacement
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum InlineReplacement {
    /// Replace text with a mix of text and author-year citations
    AuthorYearCitations,

    /// Replace text with a mix of text and bracketed numeric citations
    BracketedNumericCitations,

    /// Replace text with a mix of text and parenthetic numeric citations
    ParentheticNumericCitations,

    /// Replace text with a mix of text and superscripted numeric citations
    SuperscriptedNumericCitations,
}

/// Walks over the node collecting replacements and references
#[derive(Debug, Default)]
pub(super) struct Collector {
    /// Replacements for block nodes
    pub block_replacements: HashMap<NodeId, (BlockReplacement, Vec<Block>)>,

    /// Replacements for inline nodes
    pub inline_replacements: HashMap<NodeId, (InlineReplacement, Vec<Inline>)>,

    /// Whether currently in the References (or Bibliography) section
    in_references: bool,

    /// References collected from walking node
    pub references: Option<Vec<Reference>>,

    /// Whether references were found in an ordered (numbered) list
    pub references_are_ordered: bool,

    /// Determined citation style based on heuristics
    pub citation_style: Option<InlineReplacement>,
}

impl VisitorMut for Collector {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        if let Node::Article(Article { content, .. }) = node {
            self.visit_blocks(content);
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::Heading(heading) => self.visit_heading(heading),
            Block::Paragraph(paragraph) => self.visit_paragraph(paragraph),
            Block::List(list) => self.visit_list(list),

            // Process nested block content for figure detection
            Block::Admonition(Admonition { content, .. })
            | Block::IncludeBlock(IncludeBlock {
                content: Some(content),
                ..
            })
            | Block::Section(Section { content, .. })
            | Block::StyledBlock(StyledBlock { content, .. }) => self.visit_blocks(content),
            Block::ForBlock(ForBlock {
                content,
                iterations,
                ..
            }) => {
                self.visit_blocks(content);
                if let Some(iterations) = iterations {
                    self.visit_blocks(iterations);
                }
                WalkControl::Continue
            }

            _ => WalkControl::Continue,
        }
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if self.in_references {
            // Do not do the following if in references section since things like
            // number in brackets are normal parts of the formatting of references,
            // not citations!
            return WalkControl::Continue;
        }

        match inline {
            Inline::MathInline(math) => self.visit_math_inline(math),
            Inline::Text(text) => self.visit_text(text),
            _ => {}
        }

        WalkControl::Continue
    }
}

// Detect figure captions like "Figure 1.", "Fig 2:", "Figure 12 -", etc.
static FIGURE_CAPTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(?:Figure|Fig\.?)\s*(\d+)[.:\-\s]*").expect("invalid regex"));

// Detect table captions like "Table 1.", "Table 2:", "Table 12 -", etc.
static TABLE_CAPTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(?:Table)\s*(\d+)[.:\-\s]*").expect("invalid regex"));

impl Collector {
    /// Visit a vector of blocks such as Article or Section content
    ///
    /// Detects adjacent ImageObject and figure caption pairs in the article content
    /// and creates Figure block replacements.
    fn visit_blocks(&mut self, blocks: &[Block]) -> WalkControl {
        let mut index = 0;
        while index < blocks.len().saturating_sub(1) {
            let current = &blocks[index];
            let next = &blocks[index + 1];

            // Check for ImageObject followed by caption
            if let (Block::ImageObject(image), Block::Paragraph(caption_para)) = (current, next) {
                if let Some((figure_number, cleaned_caption)) = maybe_figure_caption(caption_para) {
                    let mut figure = Figure::new(vec![current.clone()]);
                    figure.caption = Some(vec![Block::Paragraph(cleaned_caption)]);
                    figure.label = Some(figure_number);

                    // Replace first block with figure, second with empty
                    self.block_replacements.insert(
                        image.node_id(),
                        (
                            BlockReplacement::ImageThenCaption,
                            vec![Block::Figure(figure)],
                        ),
                    );
                    self.block_replacements.insert(
                        caption_para.node_id(),
                        (BlockReplacement::ImageThenCaption, vec![]),
                    );

                    index += 2; // Skip both blocks
                    continue;
                }
            }

            // Check for caption followed by ImageObject
            if let (Block::Paragraph(caption_para), Block::ImageObject(image)) = (current, next) {
                if let Some((figure_number, cleaned_caption)) = maybe_figure_caption(caption_para) {
                    let mut figure = Figure::new(vec![next.clone()]);
                    figure.caption = Some(vec![Block::Paragraph(cleaned_caption)]);
                    figure.label = Some(figure_number);

                    // Replace first block with figure, second with empty
                    self.block_replacements.insert(
                        caption_para.node_id(),
                        (
                            BlockReplacement::CaptionThenImage,
                            vec![Block::Figure(figure)],
                        ),
                    );
                    self.block_replacements.insert(
                        image.node_id(),
                        (BlockReplacement::CaptionThenImage, vec![]),
                    );

                    index += 2; // Skip both blocks
                    continue;
                }
            }

            // Check for caption followed by Table (only caption before table is considered)
            if let (Block::Paragraph(caption_para), Block::Table(table)) = (current, next) {
                if let Some((table_number, cleaned_caption)) = maybe_table_caption(caption_para) {
                    let mut new_table = table.clone();
                    new_table.caption = Some(vec![Block::Paragraph(cleaned_caption)]);
                    new_table.label = Some(table_number);

                    // Replace caption with empty and table with updated table
                    self.block_replacements.insert(
                        caption_para.node_id(),
                        (BlockReplacement::CaptionThenTable, vec![]),
                    );
                    self.block_replacements.insert(
                        table.node_id(),
                        (
                            BlockReplacement::CaptionThenTable,
                            vec![Block::Table(new_table)],
                        ),
                    );

                    index += 2; // Skip both blocks
                    continue;
                }
            }

            index += 1;
        }

        WalkControl::Continue
    }

    /// Visit a [`Heading`] node
    ///
    /// Tracks when entering or leaving the References/Bibliography section
    /// to determine if subsequent lists should be treated as reference citations.
    fn visit_heading(&mut self, heading: &Heading) -> WalkControl {
        static REFERENCES_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?i)^\s*(?:\d+\.?\s*|[a-z]\.?\s*|[ivx]+\.?\s*)?(references?|bibliography|works?\s+cited|literature\s+cited|citations?|sources?|reference\s+list|further\s+reading|additional\s+sources|for\s+further\s+information)\s*$").expect("invalid regex")
        });

        // Detect if entering references section
        let text = to_text(&heading.content).to_lowercase();
        if REFERENCES_REGEX.is_match(&text) {
            self.in_references = true;
        } else if heading.level <= 3 {
            self.in_references = false;
        }

        WalkControl::Continue
    }

    /// Visit a [`Paragraph`] node
    ///
    /// If in the references section, parses the paragraph as a [`Reference`].
    fn visit_paragraph(&mut self, paragraph: &Paragraph) -> WalkControl {
        if self.in_references {
            let text = to_text(paragraph);
            let reference = text_to_reference(&text);
            if let Some(references) = self.references.as_mut() {
                references.push(reference);
            } else {
                self.references = Some(vec![reference]);
            }
        }

        WalkControl::Continue
    }

    /// Visit a [`List`] node
    ///
    /// If in the references section, transforms the list to a set of
    /// [`Reference`]s to assign to the root node.
    fn visit_list(&mut self, list: &List) -> WalkControl {
        if self.in_references {
            let is_numeric = matches!(list.order, ListOrder::Ascending);

            // Record whether this references list is ordered/numbered
            self.references_are_ordered = is_numeric;

            let mut references = Vec::new();
            for (index, item) in list.items.iter().enumerate() {
                let text = to_text(item);
                let mut reference = text_to_reference(&text);
                // If the list is numeric then set the id to the reference
                if is_numeric {
                    reference.id = Some(format!("ref-{}", index + 1));
                }
                references.push(reference);
            }

            if !references.is_empty() {
                self.references = Some(references);
            }

            // Break walk because content in each item already processed
            WalkControl::Break
        } else {
            WalkControl::Continue
        }
    }

    /// Visit a [`MathInline`] node
    fn visit_math_inline(&mut self, math: &MathInline) {
        if let Some(inline) = bracketed_numeric_citation(&math.code) {
            self.inline_replacements.insert(
                math.node_id(),
                (InlineReplacement::BracketedNumericCitations, vec![inline]),
            );
        }

        if let Some(inline) = parenthetic_numeric_citation(&math.code) {
            self.inline_replacements.insert(
                math.node_id(),
                (InlineReplacement::ParentheticNumericCitations, vec![inline]),
            );
        }

        if let Some(inline) = superscripted_numeric_citation(&math.code) {
            self.inline_replacements.insert(
                math.node_id(),
                (
                    InlineReplacement::SuperscriptedNumericCitations,
                    vec![inline],
                ),
            );
        }
    }

    /// Visit a [`Text`] node
    fn visit_text(&mut self, text: &mut Text) {
        if let Some(inlines) = has_citations(text_with_author_year_citations(&text.value)) {
            self.inline_replacements.insert(
                text.node_id(),
                (InlineReplacement::AuthorYearCitations, inlines),
            );
        }

        if let Some(inlines) = has_citations(text_with_bracketed_numeric_citations(&text.value)) {
            self.inline_replacements.insert(
                text.node_id(),
                (InlineReplacement::BracketedNumericCitations, inlines),
            );
        }

        if let Some(inlines) = has_citations(text_with_parenthetic_numeric_citations(&text.value)) {
            self.inline_replacements.insert(
                text.node_id(),
                (InlineReplacement::ParentheticNumericCitations, inlines),
            );
        }
    }

    /// Determine the citation style of the document
    ///
    /// This method analyzes the collected references and citation replacements
    /// to decide which citation style should be used for the document.
    pub fn determine_citation_style(&mut self) {
        // Count occurrences of each citation style
        let mut style_counts = std::collections::HashMap::new();

        for (replacement_type, _) in self.inline_replacements.values() {
            style_counts
                .entry(replacement_type)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        // Determine citation style based on heuristics
        self.citation_style = if self.references_are_ordered {
            // If references are numbered, prefer numeric citations
            // Priority: bracketed > parenthetic > superscripted
            if style_counts.contains_key(&InlineReplacement::BracketedNumericCitations) {
                Some(InlineReplacement::BracketedNumericCitations)
            } else if style_counts.contains_key(&InlineReplacement::ParentheticNumericCitations) {
                Some(InlineReplacement::ParentheticNumericCitations)
            } else if style_counts.contains_key(&InlineReplacement::SuperscriptedNumericCitations) {
                Some(InlineReplacement::SuperscriptedNumericCitations)
            } else {
                // Fall back to author-year if no numeric citations found
                style_counts
                    .get(&InlineReplacement::AuthorYearCitations)
                    .map(|_| InlineReplacement::AuthorYearCitations)
            }
        } else {
            // If references are not numbered, choose the most frequent style
            style_counts
                .iter()
                .max_by_key(|&(_, &count)| count)
                .map(|(&style, _)| style.clone())
        };
    }
}

/// Detect if a paragraph matches a figure caption pattern
///
/// Returns a tuple of (figure_number, cleaned_paragraph) if the paragraph
/// starts with "Figure X" or "Fig X" where X is a number. The cleaned paragraph
/// has the figure prefix removed from the text content, handling nested inline elements.
fn maybe_figure_caption(paragraph: &Paragraph) -> Option<(String, Paragraph)> {
    let text = to_text(paragraph);

    if let Some(captures) = FIGURE_CAPTION_REGEX.captures(&text) {
        let figure_number = captures[1].to_string();
        let matched_text = captures.get(0)?.as_str();

        // Clone the paragraph and remove the matched prefix
        let mut cleaned_paragraph = paragraph.clone();
        remove_prefix_from_inlines(&mut cleaned_paragraph.content, matched_text);

        Some((figure_number, cleaned_paragraph))
    } else {
        None
    }
}

/// Detect if a paragraph matches a table caption pattern
///
/// Returns a tuple of (table_number, cleaned_paragraph) if the paragraph
/// starts with "Table X" where X is a number. The cleaned paragraph
/// has the table prefix removed from the text content, handling nested inline elements.
fn maybe_table_caption(paragraph: &Paragraph) -> Option<(String, Paragraph)> {
    let text = to_text(paragraph);

    if let Some(captures) = TABLE_CAPTION_REGEX.captures(&text) {
        let table_number = captures[1].to_string();
        let matched_text = captures.get(0)?.as_str();

        // Clone the paragraph and remove the matched prefix
        let mut cleaned_paragraph = paragraph.clone();
        remove_prefix_from_inlines(&mut cleaned_paragraph.content, matched_text);

        Some((table_number, cleaned_paragraph))
    } else {
        None
    }
}

/// Recursively remove a prefix from the beginning of a vector of inline elements
///
/// This function handles nested inline elements like Emphasis, Strong, Underline, etc. that might
/// contain the text to be removed. It modifies the inlines vector in place.
fn remove_prefix_from_inlines(inlines: &mut Vec<Inline>, prefix: &str) {
    if prefix.is_empty() || inlines.is_empty() {
        return;
    }

    // Clone the first element to avoid borrowing issues
    let first_inline = inlines[0].clone();
    match first_inline {
        Inline::Text(text_node) => {
            remove_prefix_from_text(&text_node, inlines, prefix);
        }
        Inline::Emphasis(emphasis) => {
            remove_prefix_from_nested(&emphasis.content, inlines, prefix);
        }
        Inline::Strong(strong) => {
            remove_prefix_from_nested(&strong.content, inlines, prefix);
        }
        Inline::Underline(underline) => {
            remove_prefix_from_nested(&underline.content, inlines, prefix);
        }
        Inline::Strikeout(strikeout) => {
            remove_prefix_from_nested(&strikeout.content, inlines, prefix);
        }
        Inline::Subscript(subscript) => {
            remove_prefix_from_nested(&subscript.content, inlines, prefix);
        }
        Inline::Superscript(superscript) => {
            remove_prefix_from_nested(&superscript.content, inlines, prefix);
        }
        Inline::StyledInline(styled) => {
            remove_prefix_from_nested(&styled.content, inlines, prefix);
        }
        _ => {
            // For other inline types that don't contain nested content,
            // check if they match the prefix entirely
            let inline_text = to_text(&vec![first_inline]);
            if prefix.starts_with(&inline_text) {
                let remaining_prefix = &prefix[inline_text.len()..];
                inlines.remove(0);
                remove_prefix_from_inlines(inlines, remaining_prefix);
            }
        }
    }
}

/// Handle prefix removal for text nodes
fn remove_prefix_from_text(text_node: &Text, inlines: &mut Vec<Inline>, prefix: &str) {
    if let Some(stripped) = text_node.value.strip_prefix(prefix) {
        let remaining_text = stripped.trim_start();
        if remaining_text.is_empty() {
            // Remove the entire text node if it becomes empty
            inlines.remove(0);
        } else {
            // Update the text node with the remaining text
            if let Inline::Text(ref mut text) = inlines[0] {
                text.value = remaining_text.into();
            }
        }
    } else if prefix.starts_with(&*text_node.value) {
        // The prefix spans multiple inline elements
        let remaining_prefix = &prefix[text_node.value.len()..];
        inlines.remove(0); // Remove this text node entirely
        remove_prefix_from_inlines(inlines, remaining_prefix);
    }
}

/// Handle prefix removal for inline elements with nested content
fn remove_prefix_from_nested(nested_content: &[Inline], inlines: &mut Vec<Inline>, prefix: &str) {
    let nested_text = to_text(&nested_content.to_vec());
    if prefix.starts_with(&nested_text) {
        // The prefix spans beyond this nested element
        let remaining_prefix = &prefix[nested_text.len()..];
        inlines.remove(0); // Remove this nested element entirely
        remove_prefix_from_inlines(inlines, remaining_prefix);
    } else if nested_text.starts_with(prefix) {
        // The prefix is entirely within this nested element
        // We need to get a mutable reference to the nested content
        match &mut inlines[0] {
            Inline::Emphasis(emphasis) => {
                remove_prefix_from_inlines(&mut emphasis.content, prefix);
                if emphasis.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Strong(strong) => {
                remove_prefix_from_inlines(&mut strong.content, prefix);
                if strong.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Underline(underline) => {
                remove_prefix_from_inlines(&mut underline.content, prefix);
                if underline.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Strikeout(strikeout) => {
                remove_prefix_from_inlines(&mut strikeout.content, prefix);
                if strikeout.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Subscript(subscript) => {
                remove_prefix_from_inlines(&mut subscript.content, prefix);
                if subscript.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::Superscript(superscript) => {
                remove_prefix_from_inlines(&mut superscript.content, prefix);
                if superscript.content.is_empty() {
                    inlines.remove(0);
                }
            }
            Inline::StyledInline(styled) => {
                remove_prefix_from_inlines(&mut styled.content, prefix);
                if styled.content.is_empty() {
                    inlines.remove(0);
                }
            }
            _ => {}
        }
    }
}

/// Determine if inlines contain at least one [`Citation`] or [`CitationGroup`]
fn has_citations(inlines: Vec<Inline>) -> Option<Vec<Inline>> {
    inlines
        .iter()
        .any(|inline| matches!(inline, Inline::Citation(..) | Inline::CitationGroup(..)))
        .then_some(inlines)
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use schema::shortcuts::t;

    use super::*;

    #[test]
    fn test_maybe_figure_caption() {
        use schema::shortcuts::p;

        // Valid figure captions
        let test_cases = [
            ("Figure 1. This is a caption", "1", "This is a caption"),
            ("Fig 2: Another caption", "2", "Another caption"),
            ("Figure 12 - A longer caption", "12", "A longer caption"),
            ("Fig. 5 Some caption", "5", "Some caption"),
            ("FIGURE 3. Case insensitive", "3", "Case insensitive"),
            ("figure 7: lowercase", "7", "lowercase"),
        ];

        for (input, expected_number, expected_text) in test_cases {
            let block = p([t(input)]);
            let Block::Paragraph(paragraph) = block else {
                panic!("Expected paragraph block");
            };
            let result = maybe_figure_caption(&paragraph);

            assert!(result.is_some(), "Should detect figure caption: {input}");
            let (figure_number, cleaned_paragraph) = result.expect("Should detect figure caption");
            assert_eq!(
                figure_number, expected_number,
                "Wrong figure number for: {input}"
            );

            let cleaned_text = to_text(&cleaned_paragraph);
            assert_eq!(
                cleaned_text.trim(),
                expected_text,
                "Wrong cleaned text for: {input}"
            );
        }

        // Invalid cases - should return None
        let invalid_cases = [
            "Just regular text",
            "Figure without number",
            "Fig A: with letter instead of number",
            "Not a figure caption",
            "Figure: missing number",
            "fig missing number",
        ];

        for input in invalid_cases {
            let block = p([t(input)]);
            let Block::Paragraph(paragraph) = block else {
                panic!("Expected paragraph block");
            };
            let result = maybe_figure_caption(&paragraph);
            assert!(
                result.is_none(),
                "Should not detect figure caption: {input}"
            );
        }

        // Test with complex paragraph structure
        let complex_block = p([
            t("Figure 5. This caption has "),
            schema::shortcuts::em([t("emphasis")]),
            t(" and more text."),
        ]);
        let Block::Paragraph(complex_paragraph) = complex_block else {
            panic!("Expected paragraph block");
        };
        let result = maybe_figure_caption(&complex_paragraph);
        assert!(
            result.is_some(),
            "Should handle complex paragraph structure"
        );
        let (figure_number, cleaned_paragraph) =
            result.expect("Should detect complex figure caption");
        assert_eq!(figure_number, "5");

        let cleaned_text = to_text(&cleaned_paragraph);
        assert_eq!(
            cleaned_text.trim(),
            "This caption has emphasis and more text."
        );

        // Test edge case: figure prefix is the entire first text node
        let edge_block = p([t("Figure 1. "), t("Second text node with caption.")]);
        let Block::Paragraph(edge_paragraph) = edge_block else {
            panic!("Expected paragraph block");
        };
        let result = maybe_figure_caption(&edge_paragraph);
        assert!(
            result.is_some(),
            "Should handle prefix as entire first text node"
        );
        let (figure_number, cleaned_paragraph) =
            result.expect("Should detect edge case figure caption");
        assert_eq!(figure_number, "1");

        let cleaned_text = to_text(&cleaned_paragraph);
        assert_eq!(cleaned_text.trim(), "Second text node with caption.");
    }
}
