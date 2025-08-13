use codec_text_trait::to_text;
use common::{once_cell::sync::Lazy, regex::Regex};
use schema::{
    Block, Citation, CitationGroup, Inline, List, Node, Reference, VisitorMut, WalkControl,
    WalkNode,
};

/// Add structure to a document
pub fn structuring<T: WalkNode>(node: &mut T) {
    let mut walker = Walker::default();
    node.walk_mut(&mut walker);
}

#[derive(Debug, Default)]
struct Walker {
    /// Whether currently in the References section
    in_references: bool,

    /// References collected from walking document
    references: Option<Vec<Reference>>,
}

impl VisitorMut for Walker {
    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        match node {
            Node::Article(article) => {
                self.walk(article);

                // If any references were collected then assign to article
                if let Some(references) = self.references.take() {
                    article.references = Some(references);
                }
            }

            _ => {}
        }

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::Heading(heading) => {
                // Detect if entering references section
                let text = to_text(&heading.content).to_lowercase();
                if matches!(text.trim(), "references" | "bibliography") {
                    self.in_references = true;
                } else {
                    if heading.level <= 3 {
                        self.in_references = false;
                    }
                }
            }

            Block::List(list) => {
                if self.in_references {
                    self.references = list_to_references(list);
                }
            }

            _ => {}
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        match inline {
            Inline::MathInline(math) => {
                // Detect superscript with empty base as produced by some OCR
                // for citations.
                static SUPERSCRIPT_REGEX: Lazy<Regex> =
                    Lazy::new(|| Regex::new(r"\{\s*\}\^\{(.*?)\}").expect("invalid regex"));
                if let Some(captures) = SUPERSCRIPT_REGEX.captures(&math.code) {
                    if let Some(sequence) = maybe_citation_sequence(&captures[1]) {
                        *inline = citation_sequence_to_inline(sequence);
                    }
                }
            }

            _ => {}
        }

        WalkControl::Continue
    }
}

/// Detect if a string matches a sequence of citation numbers separated by commas and dashes
///
/// Returns a vector of numbers, commas, and dashes if the string only contains those,
/// commas and dashes are always between numbers, and all numbers are less than 500.
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

                // Check if the current number is valid (< 500)
                if let Ok(num) = current_number.parse::<u32>() {
                    if num >= 500 {
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
            if num >= 500 {
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
        Inline::Citation(Citation::new(sequence.swap_remove(0)))
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
                    citations.push(Citation::new(num.to_string()));
                }

                // Skip the dash and end number
                index += 3;

                // Skip comma if present
                if index < sequence.len() && sequence[index] == "," {
                    index += 1;
                }
            } else {
                // Single number
                citations.push(Citation::new(current.clone()));
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

/// Transform a [`List`] to a set of [`Reference`]s to assign to the root node
fn list_to_references(list: &List) -> Option<Vec<Reference>> {
    let mut references = Vec::new();
    for (index, item) in list.items.iter().enumerate() {
        let text = to_text(item);
        if let Some(reference) = codec_biblio::decode::text(&text)
            .ok()
            .and_then(|mut refs| refs.pop())
        {
            references.push(Reference {
                id: Some((index + 1).to_string()),
                ..reference
            });
        };
    }
    (!references.is_empty()).then_some(references)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maybe_citation_sequence() {
        // Valid cases
        let valid_cases = [
            // Single numbers
            ("1", vec!["1"]),
            ("42", vec!["42"]),
            ("499", vec!["499"]),
            ("0", vec!["0"]),
            // Comma separated
            ("1,2,3", vec!["1", ",", "2", ",", "3"]),
            ("5,10,15", vec!["5", ",", "10", ",", "15"]),
            ("0,1", vec!["0", ",", "1"]),
            // Dash separated
            ("1-3", vec!["1", "-", "3"]),
            ("10-20", vec!["10", "-", "20"]),
            ("0-2", vec!["0", "-", "2"]),
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
                "Failed for input: {}",
                input
            );
        }

        // Invalid cases
        let invalid_cases = [
            // Empty/whitespace only
            "", "   ", // Non-numeric characters
            "a", "1a", "1,a", "1.2", "1,2,abc", // Invalid separators
            ",1", "1,", "-1", "1-", "1,,2", "1--2", "1,-2", "1-,2", // Numbers >= 500
            "500", "1000", "1,500", "499,500", // Other characters
            "1;2", "1:2", "1&2", "1+2", "1/2", "1|2",
        ];

        for input in invalid_cases {
            assert_eq!(
                maybe_citation_sequence(input),
                None,
                "Should be None for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_citation_sequence_to_inline() {
        // Test single citation
        let single_sequence = vec!["5".to_string()];
        match citation_sequence_to_inline(single_sequence) {
            Inline::Citation(citation) => {
                assert_eq!(citation.target, "5");
            }
            _ => panic!("Expected Citation for single element"),
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
                assert_eq!(group.items[0].target, "1");
                assert_eq!(group.items[1].target, "3");
                assert_eq!(group.items[2].target, "7");
            }
            _ => panic!("Expected CitationGroup for comma-separated"),
        }

        // Test range expansion
        let range_sequence = vec!["1".to_string(), "-".to_string(), "3".to_string()];
        match citation_sequence_to_inline(range_sequence) {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 3);
                assert_eq!(group.items[0].target, "1");
                assert_eq!(group.items[1].target, "2");
                assert_eq!(group.items[2].target, "3");
            }
            _ => panic!("Expected CitationGroup for range"),
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
                assert_eq!(group.items[0].target, "1");
                assert_eq!(group.items[1].target, "3");
                assert_eq!(group.items[2].target, "4");
                assert_eq!(group.items[3].target, "5");
                assert_eq!(group.items[4].target, "7");
            }
            _ => panic!("Expected CitationGroup for mixed sequence"),
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
                assert_eq!(group.items[0].target, "2");
                assert_eq!(group.items[1].target, "3");
                assert_eq!(group.items[2].target, "4");
                assert_eq!(group.items[3].target, "8");
                assert_eq!(group.items[4].target, "10");
                assert_eq!(group.items[5].target, "11");
                assert_eq!(group.items[6].target, "12");
            }
            _ => panic!("Expected CitationGroup for multi-range sequence"),
        }
    }
}
