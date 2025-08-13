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
        if let Node::Article(article) = node {
            self.walk(article);

            // If any references were collected then assign to article
            if let Some(references) = self.references.take() {
                article.references = Some(references);
            }
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
                } else if heading.level <= 3 {
                    self.in_references = false;
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
        // These citation regexes all capture `[\d+\,\-\s]+` (digits, commas, dashes, spaces) but note that
        // `maybe_citation_sequence` also checks for correct arrangement of those

        // Detect square brackets containing only numbers, commas and dashes as
        // produced by some OCR for bracketed citations as used in Vancouver, IEEE citation styles
        static BRACKETS_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\[([\d+\,\-\s]+)\]").expect("invalid regex"));

        // Detect parentheses containing only numbers, commas and dashes as
        // produced by some OCR of citations
        static PARENS_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\(([\d+\,\-\s]+)\)").expect("invalid regex"));

        // Detect superscript with empty base as produced by some OCR for
        // superscript citations as used in ACS, AMA, Chicago citation
        // styles
        static MATH_SUP_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\{\s*\}\^\{([\d+\,\-\s]+)\}").expect("invalid regex"));

        match inline {
            Inline::MathInline(math) => {
                if let Some(captures) = MATH_SUP_REGEX
                    .captures(&math.code)
                    .or(BRACKETS_REGEX.captures(&math.code))
                    .or(PARENS_REGEX.captures(&math.code))
                {
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
                id: Some(format!("ref-{}", index + 1)),
                ..reference
            });
        };
    }
    (!references.is_empty()).then_some(references)
}

#[cfg(test)]
mod tests {
    use common::eyre::{Result, bail};
    use common_dev::pretty_assertions::assert_eq;
    use schema::{
        Article,
        shortcuts::{ct, ctg, h1, li, mi, ol, p, t},
    };

    use super::*;

    #[test]
    fn test_reference_list_to_references() -> Result<()> {
        // Single reference with DOI
        let mut article = Node::Article(Article::new(vec![
            h1([t("References")]),
            ol([li([t(
                "Author, A. B., & Author, C. D. (Year). Title of article. Journal Name, Volume(Issue), pages. 10.0000/xyz",
            )])]),
        ]));
        structuring(&mut article);
        let Node::Article(Article {
            references: Some(refs),
            ..
        }) = article
        else {
            bail!("Should have references")
        };
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].doi, Some("10.0000/xyz".into()));
        assert_eq!(refs[0].id, Some("ref-1".into()));

        // Multiple references with sequential IDs
        let mut article = Node::Article(Article::new(vec![
            h1([t("References")]),
            ol([
                li([t(
                    "First Author. (2020). First paper. Journal A, 1(1), 1-10.",
                )]),
                li([t(
                    "Second Author. (2021). Second paper. Journal B, 2(2), 11-20.",
                )]),
                li([t(
                    "Third Author. (2022). Third paper. Journal C, 3(3), 21-30.",
                )]),
            ]),
        ]));
        structuring(&mut article);
        let Node::Article(Article {
            references: Some(refs),
            ..
        }) = article
        else {
            bail!("Should have references")
        };
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].id, Some("ref-1".into()));
        assert_eq!(refs[1].id, Some("ref-2".into()));
        assert_eq!(refs[2].id, Some("ref-3".into()));

        // "Bibliography" heading should also trigger reference detection
        let mut article = Node::Article(Article::new(vec![
            h1([t("Bibliography")]),
            ol([li([t(
                "Author, A. (2023). Test paper. Test Journal, 1, 1-5.",
            )])]),
        ]));
        structuring(&mut article);
        let Node::Article(Article {
            references: Some(refs),
            ..
        }) = article
        else {
            bail!("Should have references for Bibliography heading")
        };
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, Some("ref-1".into()));

        // Case insensitive heading detection
        let mut article = Node::Article(Article::new(vec![
            h1([t("REFERENCES")]),
            ol([li([t(
                "Author, A. (2023). Test paper. Test Journal, 1, 1-5.",
            )])]),
        ]));
        structuring(&mut article);
        let Node::Article(Article {
            references: Some(refs),
            ..
        }) = article
        else {
            bail!("Should have references for uppercase heading")
        };
        assert_eq!(refs.len(), 1);

        // No references section should result in no references
        let mut article = Node::Article(Article::new(vec![
            h1([t("Introduction")]),
            p([t("This is just content.")]),
        ]));
        structuring(&mut article);
        let Node::Article(Article { references, .. }) = article else {
            bail!("Should be an article")
        };
        assert!(references.is_none());

        // Empty reference list should result in no references
        let mut article = Node::Article(Article::new(vec![h1([t("References")]), ol([])]));
        structuring(&mut article);
        let Node::Article(Article { references, .. }) = article else {
            bail!("Should be an article")
        };
        assert!(references.is_none());

        // References section should reset when encountering other high-level headings
        let mut article = Node::Article(Article::new(vec![
            h1([t("References")]),
            ol([li([t(
                "First Author. (2020). First paper. Journal A, 1(1), 1-10.",
            )])]),
            h1([t("Conclusion")]),
            ol([li([t("This should not be treated as a reference")])]),
        ]));
        structuring(&mut article);
        let Node::Article(Article {
            references: Some(refs),
            ..
        }) = article
        else {
            bail!("Should have references")
        };
        assert_eq!(refs.len(), 1);

        // Multiple reference sections should use the last one
        let mut article = Node::Article(Article::new(vec![
            h1([t("References")]),
            ol([li([t(
                "First Author. (2020). First paper. Journal A, 1(1), 1-10.",
            )])]),
            h1([t("Additional References")]),
            h1([t("Bibliography")]),
            ol([
                li([t(
                    "Second Author. (2021). Second paper. Journal B, 2(2), 11-20.",
                )]),
                li([t(
                    "Third Author. (2022). Third paper. Journal C, 3(3), 21-30.",
                )]),
            ]),
        ]));
        structuring(&mut article);
        let Node::Article(Article {
            references: Some(refs),
            ..
        }) = article
        else {
            bail!("Should have references")
        };
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].id, Some("ref-1".into()));
        assert_eq!(refs[1].id, Some("ref-2".into()));

        Ok(())
    }

    #[test]
    fn test_math_inline_to_citation() {
        // Simple superscript citation
        let mut node = p([mi("{ }^{1}", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([ct("ref-1")]));

        // Range expansion in superscript
        let mut node = p([mi("{ }^{1-3}", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([ctg(["ref-1", "ref-2", "ref-3"])]));

        // Bracketed citation
        let mut node = p([mi("[5]", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([ct("ref-5")]));

        // Parentheses citation
        let mut node = p([mi("(7)", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([ct("ref-7")]));

        // Comma-separated citations in brackets
        let mut node = p([mi("[1,3,5]", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([ctg(["ref-1", "ref-3", "ref-5"])]));

        // Mixed range and individual citations
        let mut node = p([mi("{ }^{2-4,8}", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([ctg(["ref-2", "ref-3", "ref-4", "ref-8"])]));

        // Citations with whitespace in parentheses
        let mut node = p([mi("( 10 , 12 )", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([ctg(["ref-10", "ref-12"])]));

        // Complex range with multiple parts
        let mut node = p([mi("[15-17,20,25-27]", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(
            node,
            p([ctg([
                "ref-15", "ref-16", "ref-17", "ref-20", "ref-25", "ref-26", "ref-27"
            ])])
        );

        // Invalid citation (contains zero) should not be converted
        let mut node = p([mi("{ }^{0,1}", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([mi("{ }^{0,1}", Some("tex"))]));

        //  Invalid citation (contains letters) should not be converted
        let mut node = p([mi("[1a,2]", Some("tex"))]);
        structuring(&mut node);
        assert_eq!(node, p([mi("[1a,2]", Some("tex"))]));
    }

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
