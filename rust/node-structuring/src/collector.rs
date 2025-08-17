use std::collections::HashMap;

use codec_biblio::decode::text_to_reference;
use codec_text_trait::to_text;
use common::{once_cell::sync::Lazy, regex::Regex};
use schema::{
    Admonition, Article, Block, Citation, CitationGroup, Figure, ForBlock, Heading, IncludeBlock,
    Inline, List, ListOrder, MathInline, Node, NodeId, Paragraph, Reference, Section, StyledBlock,
    Text, VisitorMut, WalkControl, shortcuts::t,
};

/// Walks over the node collecting replacements and references
#[derive(Debug, Default)]
pub(super) struct Collector {
    /// Replacements for block nodes
    pub block_replacements: HashMap<NodeId, Vec<Block>>,

    /// Replacements for inline nodes
    pub inline_replacements: HashMap<NodeId, Vec<Inline>>,

    /// Whether currently in the References (or Bibliography) section
    in_references: bool,

    /// References collected from walking node
    pub references: Option<Vec<Reference>>,
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

// These citation regexes all capture `[\d+\,\-–—\s]+` (digits, commas, hyphens, en/em dashes, spaces) but note that
// `maybe_citation_sequence` also checks for correct arrangement of those

// Detect square brackets containing only numbers, commas and dashes as
// produced by some OCR for bracketed citations as used in Vancouver, IEEE citation styles
static CITE_BRACKETS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([\d+\,\-–—\s]+)\]").expect("invalid regex"));

// Detect parentheses containing only numbers, commas and dashes as
// produced by some OCR of citations
static CITE_PARENS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\(([\d+\,\-–—\s]+)\)").expect("invalid regex"));

// Detect superscript with empty base as produced by some OCR for
// superscript citations as used in ACS, AMA, Chicago citation
// styles
static CITE_MATH_SUP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\s*\}\^\{([\d+\,\-–—\s]+)\}").expect("invalid regex"));

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
                    self.block_replacements
                        .insert(image.node_id(), vec![Block::Figure(figure)]);
                    self.block_replacements
                        .insert(caption_para.node_id(), vec![]);

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
                    self.block_replacements
                        .insert(caption_para.node_id(), vec![Block::Figure(figure)]);
                    self.block_replacements.insert(image.node_id(), vec![]);

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
                    self.block_replacements
                        .insert(caption_para.node_id(), vec![]);
                    self.block_replacements
                        .insert(table.node_id(), vec![Block::Table(new_table)]);

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
    ///
    /// Detects citation patterns in math expressions like superscript citations
    /// and converts them to structured Citation/CitationGroup replacements.
    fn visit_math_inline(&mut self, math: &MathInline) {
        if let Some(captures) = CITE_MATH_SUP_REGEX
            .captures(&math.code)
            .or(CITE_BRACKETS_REGEX.captures(&math.code))
            .or(CITE_PARENS_REGEX.captures(&math.code))
        {
            if let Some(sequence) = maybe_citation_sequence(&captures[1]) {
                self.inline_replacements
                    .insert(math.node_id(), vec![citation_sequence_to_inline(sequence)]);
            }
        }
    }

    /// Visit a [`Text`] node
    ///
    /// Scans plain text for bracketed and parenthetical citation patterns
    /// and replaces them with structured Citation/CitationGroup nodes.
    fn visit_text(&mut self, text: &mut Text) {
        let mut replacements: Vec<Inline> = Vec::new();
        let mut last_end = 0;
        let text_value = &text.value;

        // Collect all valid citation matches with their positions
        let mut matches: Vec<(usize, usize, Vec<String>)> = Vec::new();

        // Find bracket citations [1,2,3]
        for capture in CITE_BRACKETS_REGEX.captures_iter(text_value) {
            if let Some(m) = capture.get(0) {
                if let Some(sequence) = maybe_citation_sequence(&capture[1]) {
                    matches.push((m.start(), m.end(), sequence));
                }
            }
        }

        // Find parentheses citations (1,2,3)
        for capture in CITE_PARENS_REGEX.captures_iter(text_value) {
            if let Some(m) = capture.get(0) {
                if let Some(sequence) = maybe_citation_sequence(&capture[1]) {
                    matches.push((m.start(), m.end(), sequence));
                }
            }
        }

        // Sort matches by position to process them in order
        matches.sort_by_key(|&(start, _, _)| start);

        // Process each valid match
        for (start, end, sequence) in matches {
            // Skip overlapping matches
            if start < last_end {
                continue;
            }

            // Add text before this match
            if start > last_end {
                let before_text = &text_value[last_end..start];
                if !before_text.is_empty() {
                    replacements.push(t(before_text));
                }
            }

            // Add the citation
            replacements.push(citation_sequence_to_inline(sequence));

            last_end = end;
        }

        // Add any remaining text after the last match
        if last_end < text_value.len() {
            let remaining_text = &text_value[last_end..];
            if !remaining_text.is_empty() {
                replacements.push(t(remaining_text));
            }
        }

        // If we found any citations, replace this text node with the replacement vector
        if replacements.len() > 1 {
            self.inline_replacements
                .insert(text.node_id(), replacements);
        }
    }
}

/// Detect if a string matches a sequence of citation numbers separated by commas and dashes
///
/// Returns a vector of numbers, commas, and dashes if the string only contains those,
/// commas and dashes are always between numbers, and all numbers are greater than 0 and less than 500.
/// Supports hyphens (-), en dashes (–), and em dashes (—) as range separators.
/// Return `None` otherwise.
fn maybe_citation_sequence(string: &str) -> Option<Vec<String>> {
    let mut sequence = Vec::new();
    let mut current_number = String::new();
    let mut last_was_separator = false;
    let mut expecting_number = true;

    for ch in string.chars() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                // Ignore whitespace
                continue;
            }
            '0'..='9' => {
                current_number.push(ch);
                last_was_separator = false;
                expecting_number = false;
            }
            ',' | '-' | '–' | '—' => {
                if current_number.is_empty() || last_was_separator || expecting_number {
                    return None;
                }

                // Check if the current number is valid (> 0 and < 500)
                if let Ok(num) = current_number.parse::<u32>() {
                    if num == 0 || num >= 500 {
                        return None;
                    }
                } else {
                    return None;
                }

                sequence.push(current_number.clone());
                // Normalize all dash types to hyphen for consistent processing
                let separator = if matches!(ch, '–' | '—') {
                    "-"
                } else {
                    &ch.to_string()
                };
                sequence.push(separator.to_string());
                current_number.clear();
                last_was_separator = true;
                expecting_number = true;
            }
            _ => {
                // Any other character makes this not a citation sequence
                return None;
            }
        }
    }

    // Handle the last number if there is one
    if !current_number.is_empty() {
        if let Ok(num) = current_number.parse::<u32>() {
            if num == 0 || num >= 500 {
                return None;
            }
            sequence.push(current_number);
        } else {
            return None;
        }
    } else if last_was_separator {
        // String ends with a separator, which is invalid
        return None;
    }

    (!sequence.is_empty()).then_some(sequence)
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
            handle_text_prefix_removal(&text_node, inlines, prefix);
        }
        Inline::Emphasis(emphasis) => {
            handle_nested_inline_prefix_removal(&emphasis.content, inlines, prefix);
        }
        Inline::Strong(strong) => {
            handle_nested_inline_prefix_removal(&strong.content, inlines, prefix);
        }
        Inline::Underline(underline) => {
            handle_nested_inline_prefix_removal(&underline.content, inlines, prefix);
        }
        Inline::Strikeout(strikeout) => {
            handle_nested_inline_prefix_removal(&strikeout.content, inlines, prefix);
        }
        Inline::Subscript(subscript) => {
            handle_nested_inline_prefix_removal(&subscript.content, inlines, prefix);
        }
        Inline::Superscript(superscript) => {
            handle_nested_inline_prefix_removal(&superscript.content, inlines, prefix);
        }
        Inline::StyledInline(styled) => {
            handle_nested_inline_prefix_removal(&styled.content, inlines, prefix);
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
fn handle_text_prefix_removal(text_node: &Text, inlines: &mut Vec<Inline>, prefix: &str) {
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
fn handle_nested_inline_prefix_removal(
    nested_content: &[Inline],
    inlines: &mut Vec<Inline>,
    prefix: &str,
) {
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

/// Transform a sequence of citation numbers, commas and dashes into a
/// [`Citation`] or [`CitationGroup`]
///
/// Dashes, indicate a range to be expanded, e.g.
///
/// 1-3      => 1, 2, 3
/// 1,3-5,7  => 1, 3, 4, 5, 7
fn citation_sequence_to_inline(mut sequence: Vec<String>) -> Inline {
    if sequence.len() == 1 {
        let num = sequence.swap_remove(0);
        Inline::Citation(Citation::new(format!("ref-{num}")))
    } else {
        let mut citations = Vec::new();
        let mut index = 0;

        while index < sequence.len() {
            let current = &sequence[index];

            // Check if this is a number followed by a dash
            if index + 2 < sequence.len() && sequence[index + 1] == "-" {
                let start_num = current.parse::<u32>().expect("should be valid number");
                let end_num = sequence[index + 2]
                    .parse::<u32>()
                    .expect("should be valid number");

                // Expand the range
                for num in start_num..=end_num {
                    citations.push(Citation::new(format!("ref-{num}")));
                }

                // Skip the dash and end number
                index += 3;

                // Skip comma if present
                if index < sequence.len() && sequence[index] == "," {
                    index += 1;
                }
            } else {
                // Single number
                citations.push(Citation::new(format!("ref-{current}")));
                index += 1;

                // Skip comma if present
                if index < sequence.len() && sequence[index] == "," {
                    index += 1;
                }
            }
        }

        Inline::CitationGroup(CitationGroup::new(citations))
    }
}

#[cfg(test)]
mod tests {
    use common::eyre::{Result, bail};
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_maybe_citation_sequence() {
        // Valid cases
        let valid_cases = [
            // Single numbers
            ("1", vec!["1"]),
            ("42", vec!["42"]),
            ("499", vec!["499"]),
            // Comma separated
            ("1,2,3", vec!["1", ",", "2", ",", "3"]),
            ("5,10,15", vec!["5", ",", "10", ",", "15"]),
            // Dash separated (hyphen, en dash, em dash)
            ("1-3", vec!["1", "-", "3"]),
            ("10-20", vec!["10", "-", "20"]),
            ("1–3", vec!["1", "-", "3"]), // en dash normalized to hyphen
            ("10—20", vec!["10", "-", "20"]), // em dash normalized to hyphen
            // Mixed separators
            ("1,3-5,7", vec!["1", ",", "3", "-", "5", ",", "7"]),
            ("1,3–5,7", vec!["1", ",", "3", "-", "5", ",", "7"]), // en dash
            ("1,3—5,7", vec!["1", ",", "3", "-", "5", ",", "7"]), // em dash
            (
                "2-4,8,10-12",
                vec!["2", "-", "4", ",", "8", ",", "10", "-", "12"],
            ),
            (
                "2–4,8,10—12", // mixed dash types
                vec!["2", "-", "4", ",", "8", ",", "10", "-", "12"],
            ),
            // With whitespace
            ("1, 2, 3", vec!["1", ",", "2", ",", "3"]),
            (" 1 - 3 ", vec!["1", "-", "3"]),
            ("\t1,\n3-5,\r7\t", vec!["1", ",", "3", "-", "5", ",", "7"]),
        ];

        for (input, expected) in valid_cases {
            let expected_strings: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
            assert_eq!(
                maybe_citation_sequence(input),
                Some(expected_strings),
                "Failed for input: {input}"
            );
        }

        // Invalid cases
        let invalid_cases = [
            // Empty/whitespace only
            "", "   ", // Non-numeric characters
            "a", "1a", "1,a", "1.2", "1,2,abc", // Invalid separators
            ",1", "1,", "-1", "1-", "1,,2", "1--2", "1,-2", "1-,2", // Numbers <= 0 or >= 500
            "0", "500", "1000", "1,500", "499,500", "0,1", "0-2", // Other characters
            "1;2", "1:2", "1&2", "1+2", "1/2", "1|2",
        ];

        for input in invalid_cases {
            assert_eq!(
                maybe_citation_sequence(input),
                None,
                "Should be None for input: {input}"
            );
        }
    }

    #[test]
    fn test_citation_sequence_to_inline() -> Result<()> {
        // Test single citation
        let single_sequence = vec!["5".to_string()];
        match citation_sequence_to_inline(single_sequence) {
            Inline::Citation(citation) => {
                assert_eq!(citation.target, "ref-5");
            }
            _ => bail!("Expected Citation for single element"),
        }

        // Test comma-separated citations
        let comma_sequence = vec![
            "1".to_string(),
            ",".to_string(),
            "3".to_string(),
            ",".to_string(),
            "7".to_string(),
        ];
        match citation_sequence_to_inline(comma_sequence) {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 3);
                assert_eq!(group.items[0].target, "ref-1");
                assert_eq!(group.items[1].target, "ref-3");
                assert_eq!(group.items[2].target, "ref-7");
            }
            _ => bail!("Expected CitationGroup for comma-separated"),
        }

        // Test range expansion
        let range_sequence = vec!["1".to_string(), "-".to_string(), "3".to_string()];
        match citation_sequence_to_inline(range_sequence) {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 3);
                assert_eq!(group.items[0].target, "ref-1");
                assert_eq!(group.items[1].target, "ref-2");
                assert_eq!(group.items[2].target, "ref-3");
            }
            _ => bail!("Expected CitationGroup for range"),
        }

        // Test mixed: 1,3-5,7 => 1, 3, 4, 5, 7
        let mixed_sequence = vec![
            "1".to_string(),
            ",".to_string(),
            "3".to_string(),
            "-".to_string(),
            "5".to_string(),
            ",".to_string(),
            "7".to_string(),
        ];
        match citation_sequence_to_inline(mixed_sequence) {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 5);
                assert_eq!(group.items[0].target, "ref-1");
                assert_eq!(group.items[1].target, "ref-3");
                assert_eq!(group.items[2].target, "ref-4");
                assert_eq!(group.items[3].target, "ref-5");
                assert_eq!(group.items[4].target, "ref-7");
            }
            _ => bail!("Expected CitationGroup for mixed sequence"),
        }

        // Test multiple ranges: 2-4,8,10-12
        let multi_range_sequence = vec![
            "2".to_string(),
            "-".to_string(),
            "4".to_string(),
            ",".to_string(),
            "8".to_string(),
            ",".to_string(),
            "10".to_string(),
            "-".to_string(),
            "12".to_string(),
        ];
        match citation_sequence_to_inline(multi_range_sequence) {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 7); // 2,3,4,8,10,11,12
                assert_eq!(group.items[0].target, "ref-2");
                assert_eq!(group.items[1].target, "ref-3");
                assert_eq!(group.items[2].target, "ref-4");
                assert_eq!(group.items[3].target, "ref-8");
                assert_eq!(group.items[4].target, "ref-10");
                assert_eq!(group.items[5].target, "ref-11");
                assert_eq!(group.items[6].target, "ref-12");
            }
            _ => bail!("Expected CitationGroup for multi-range sequence"),
        }

        Ok(())
    }

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
