use std::collections::HashMap;

use codec_text_trait::to_text;
use common::{once_cell::sync::Lazy, regex::Regex};
use schema::{
    Block, Citation, CitationGroup, Heading, Inline, List, MathInline, NodeId, Reference, Text,
    VisitorMut, WalkControl, shortcuts::t,
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
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::Heading(heading) => self.visit_heading(heading),
            Block::List(list) => self.visit_list(list),
            _ => {}
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::MathInline(math) => self.visit_math_inline(math),
            Inline::Text(text) => self.visit_text(text),
            _ => {}
        }

        WalkControl::Continue
    }
}

// These citation regexes all capture `[\d+\,\-\s]+` (digits, commas, dashes, spaces) but note that
// `maybe_citation_sequence` also checks for correct arrangement of those

// Detect square brackets containing only numbers, commas and dashes as
// produced by some OCR for bracketed citations as used in Vancouver, IEEE citation styles
static CITE_BRACKETS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([\d+\,\-\s]+)\]").expect("invalid regex"));

// Detect parentheses containing only numbers, commas and dashes as
// produced by some OCR of citations
static CITE_PARENS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\(([\d+\,\-\s]+)\)").expect("invalid regex"));

// Detect superscript with empty base as produced by some OCR for
// superscript citations as used in ACS, AMA, Chicago citation
// styles
static CITE_MATH_SUP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\s*\}\^\{([\d+\,\-\s]+)\}").expect("invalid regex"));

impl Collector {
    /// Visit a [`Heading`] node
    ///
    /// Tracks when entering or leaving the References/Bibliography section
    /// to determine if subsequent lists should be treated as reference citations.
    fn visit_heading(&mut self, heading: &Heading) {
        // Detect if entering references section
        let text = to_text(&heading.content).to_lowercase();
        if matches!(text.trim(), "references" | "bibliography") {
            self.in_references = true;
        } else if heading.level <= 3 {
            self.in_references = false;
        }
    }

    /// Visit a [`List`] node
    ///
    /// If in the references section, transforms the list to a set of
    /// [`Reference`]s to assign to the root node.
    fn visit_list(&mut self, list: &List) {
        if self.in_references {
            let mut references = Vec::new();
            for (index, item) in list.items.iter().enumerate() {
                let text = to_text(item);
                if let Some(reference) = codec_biblio::decode::text(&text)
                    .ok()
                    .and_then(|mut refs| refs.pop())
                {
                    references.push(Reference {
                        id: Some(format!("ref-{}", index + 1)),
                        ..reference
                    });
                };
            }

            if !references.is_empty() {
                self.references = Some(references);
            }
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

        // Collect all matches from both regexes with their positions
        let mut matches: Vec<(usize, usize, String)> = Vec::new();

        // Find bracket citations [1,2,3]
        for capture in CITE_BRACKETS_REGEX.captures_iter(text_value) {
            if let Some(m) = capture.get(0) {
                matches.push((m.start(), m.end(), capture[1].to_string()));
            }
        }

        // Find parentheses citations (1,2,3)
        for capture in CITE_PARENS_REGEX.captures_iter(text_value) {
            if let Some(m) = capture.get(0) {
                matches.push((m.start(), m.end(), capture[1].to_string()));
            }
        }

        // Sort matches by position to process them in order
        matches.sort_by_key(|&(start, _, _)| start);

        // Process each match
        for (start, end, citation_text) in matches {
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

            // Check if this is a valid citation sequence
            if let Some(sequence) = maybe_citation_sequence(&citation_text) {
                replacements.push(citation_sequence_to_inline(sequence));
            } else {
                // If not a valid citation, keep the original text
                replacements.push(t(&text_value[start..end]));
            }

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
            ',' | '-' => {
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
                sequence.push(ch.to_string());
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
            // Dash separated
            ("1-3", vec!["1", "-", "3"]),
            ("10-20", vec!["10", "-", "20"]),
            // Mixed separators
            ("1,3-5,7", vec!["1", ",", "3", "-", "5", ",", "7"]),
            (
                "2-4,8,10-12",
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
}
