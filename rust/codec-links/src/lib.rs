use inflector::Inflector;
use winnow::{
    Parser, Result as ParserResult,
    ascii::{Caseless, multispace0, multispace1},
    combinator::{alt, not, preceded, repeat},
    token::{any, take_while},
};

use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, NodeType, async_trait,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{Inline, Link, Node, Paragraph, shortcuts::t},
    stencila_status::Status,
};

/// A codec for links within a document
///
/// Parses both internal links like "Figure 1", "Table 2", "Appendix A", "Equation 3"
/// and external HTTP/HTTPS URLs from text.
///
/// For figure and table links, letter suffixes are stripped from targets to point to
/// the main element (e.g., "Figure 1B" links to "#fig-1", not "#fig-1b").
pub struct LinksCodec;

#[async_trait]
impl Codec for LinksCodec {
    fn name(&self) -> &str {
        "links"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Text => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        match node_type {
            NodeType::Link => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    async fn from_str(
        &self,
        text: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        Ok((
            Node::Paragraph(Paragraph::new(decode_inlines(text))),
            DecodeInfo::none(),
        ))
    }
}

/// Decode [`Inline::Link`] and [`Inline::Text`] from plain text
///
/// Identifies both internal links (Figure 1, Table 2, etc.) and external HTTP/HTTPS URLs.
/// Figure and table links strip letter suffixes from targets (e.g., "1B" → "#fig-1").
pub fn decode_inlines(text: &str) -> Vec<Inline> {
    match text_with_links.parse(text) {
        Ok(result) => result,
        Err(_) => vec![t(text)],
    }
}

/// Parse links within surrounding text
fn text_with_links(input: &mut &str) -> ParserResult<Vec<Inline>> {
    repeat(
        1..,
        alt((
            internal_link.map(Inline::Link),
            url_link.map(Inline::Link),
            preceded(not(alt((internal_link, url_link))), any)
                .take()
                .map(|text: &str| t(text)),
        )),
    )
    .map(fold_text_inlines)
    .parse_next(input)
}

/// Parse an internal link (Figure, Table, Appendix, Equation)
fn internal_link(input: &mut &str) -> ParserResult<Link> {
    (
        alt((
            (Caseless("figure"), multispace1),
            (Caseless("fig."), multispace0),
            ("Fig", multispace0),
            (Caseless("table"), multispace1),
            (Caseless("appendix"), multispace1),
            (Caseless("app."), multispace0),
            (Caseless("equation"), multispace1),
            (Caseless("eqn"), multispace1),
            (Caseless("eqn."), multispace0),
        ))
        .take(),
        take_while(1.., |c: char| c.is_alphanumeric() || c == '-' || c == '_'),
    )
        .verify(|(_, label): &(&str, &str)| is_valid_reference_label(label))
        .map(|(label_type, label): (&str, &str)| {
            // For figures and tables, strip letter suffixes from the target
            // but preserve the original label in the content
            let target_label = if label_type.to_lowercase().starts_with("fig")
                || label_type.to_lowercase().starts_with("tab")
            {
                // Remove letter suffixes after numbers (e.g., "1B" -> "1", "2A" -> "2")
                strip_letter_suffix(label)
            } else {
                label.to_string()
            };

            let id = target_label.to_kebab_case();

            let target = if label_type.to_lowercase().starts_with("fig") {
                ["#fig-", &id].concat()
            } else if label_type.to_lowercase().starts_with("tab") {
                ["#tab-", &id].concat()
            } else if label_type.to_lowercase().starts_with("app") {
                ["#app-", &id].concat()
            } else if label_type.to_lowercase().starts_with("eq") {
                ["#eqn-", &id].concat()
            } else {
                String::new()
            };

            let content = vec![t([label_type, label].concat())];

            Link {
                target,
                content,
                ..Default::default()
            }
        })
        .parse_next(input)
}

/// Validate that a reference label contains at least one digit and no more than 3 trailing letters
/// This helps avoid false positives like "This table describes" or "See figure below"
/// Special exception: single letters (A, B, C, etc.) are valid for appendices
fn is_valid_reference_label(label: &str) -> bool {
    // Single letters are valid (for appendices like "Appendix A")
    if label.len() == 1
        && let Some(ch) = label.chars().next()
        && ch.is_alphabetic()
    {
        return true;
    }

    // Word numbers followed by hyphen and letter are valid (e.g., "Two-A")
    let word_numbers = [
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];
    let lower_label = label.to_lowercase();
    for word_num in word_numbers {
        if lower_label.starts_with(word_num)
            && lower_label.len() > word_num.len() + 1
            && lower_label.chars().nth(word_num.len()) == Some('-')
        {
            let suffix = &lower_label[word_num.len() + 1..];
            // Allow up to 3 letters after hyphen
            if suffix.len() <= 3 && suffix.chars().all(|c| c.is_alphabetic()) {
                return true;
            }
        }
    }

    // Must contain at least one digit otherwise
    let has_digit = label.chars().any(|c| c.is_ascii_digit());
    if !has_digit {
        return false;
    }

    // Count trailing characters after the last digit
    if let Some(last_digit_pos) = label
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i)
    {
        let after_digit = &label[last_digit_pos + 1..];

        if !after_digit.is_empty() {
            // Check for patterns like "1-A", "1_B" (hyphen/underscore + letters)
            if after_digit.starts_with('-') || after_digit.starts_with('_') {
                let suffix = &after_digit[1..];
                // Allow up to 3 letters after hyphen/underscore
                if suffix.len() > 3 || !suffix.chars().all(|c| c.is_alphabetic()) {
                    return false;
                }
            } else {
                // Direct letter suffixes like "1A", "1AB", "1ABC"
                if after_digit.len() > 3 || !after_digit.chars().all(|c| c.is_alphabetic()) {
                    return false;
                }
            }
        }
    }

    // Reject patterns where letters are followed by more digits (e.g., "1ABCD2")
    let chars: Vec<char> = label.chars().collect();
    for i in 0..chars.len() - 1 {
        if chars[i].is_ascii_alphabetic() && chars[i + 1].is_ascii_digit() {
            // Check if there was a digit before this letter sequence
            if (0..i).any(|j| chars[j].is_ascii_digit()) {
                return false;
            }
        }
    }

    true
}

/// Strip letter suffixes from figure/table labels to get the base number
///
/// Examples: "1B" -> "1", "2A" -> "2", "10C" -> "10", "Two-A" -> "Two", "S1" -> "S1" (no change)
fn strip_letter_suffix(label: &str) -> String {
    // Look for patterns like "number+letters" or "word-letter" at the end

    // First try: digit followed by letters at the end
    if let Some(last_digit_pos) = label
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i)
    {
        let after_digit = &label[last_digit_pos + 1..];
        if after_digit.chars().all(|c| c.is_ascii_alphabetic()) && !after_digit.is_empty() {
            return label[..last_digit_pos + 1].to_string();
        }
    }

    // Second try: hyphen followed by single letters at the end (e.g., "Two-A")
    if let Some(hyphen_pos) = label.rfind('-') {
        let after_hyphen = &label[hyphen_pos + 1..];
        if after_hyphen.len() <= 2 && after_hyphen.chars().all(|c| c.is_ascii_alphabetic()) {
            return label[..hyphen_pos].to_string();
        }
    }

    // No pattern found, return as-is
    label.to_string()
}

/// Parse a URL link (HTTP/HTTPS)
fn url_link(input: &mut &str) -> ParserResult<Link> {
    let start_pos = *input;
    let (protocol, rest): (&str, &str) = (
        alt((Caseless("https://"), Caseless("http://"))),
        take_while(1.., |c: char| {
            !c.is_whitespace() && c != ')' && c != ']' && c != '}'
        }),
    )
        .parse_next(input)?;

    // Remove trailing punctuation that's likely sentence punctuation
    let trimmed_rest = rest.trim_end_matches(['.', ',', ';', ':', '!', '?']);
    let trimmed_len = protocol.len() + trimmed_rest.len();

    // Adjust input position to not consume the trailing punctuation
    let consumed = start_pos.len() - input.len();
    if consumed > trimmed_len {
        *input = &start_pos[trimmed_len..];
    }

    let target = [protocol, trimmed_rest].concat();
    let content = vec![t(&target)];

    Ok(Link {
        target,
        content,
        ..Default::default()
    })
}

/// Helper function to fold adjacent text inlines
fn fold_text_inlines(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut folded = Vec::new();
    for inline in inlines {
        if let (Some(Inline::Text(last)), Inline::Text(text)) = (folded.last_mut(), &inline) {
            last.value.push_str(&text.value);
        } else {
            folded.push(inline);
        }
    }
    folded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_link() -> ParserResult<()> {
        // Test Figure with number
        let result = internal_link(&mut "Figure 1")?;
        assert_eq!(result.target, "#fig-1");
        assert_eq!(result.content, vec![t("Figure 1")]);

        // Test Fig. abbreviation
        let result = internal_link(&mut "Fig. 2")?;
        assert_eq!(result.target, "#fig-2");
        assert_eq!(result.content, vec![t("Fig. 2")]);

        // Test Table
        let result = internal_link(&mut "Table 5")?;
        assert_eq!(result.target, "#tab-5");
        assert_eq!(result.content, vec![t("Table 5")]);

        // Test case insensitive
        let result = internal_link(&mut "figure 10")?;
        assert_eq!(result.target, "#fig-10");
        assert_eq!(result.content, vec![t("figure 10")]);

        // Test alphanumeric label
        let result = internal_link(&mut "Figure A1")?;
        assert_eq!(result.target, "#fig-a1");
        assert_eq!(result.content, vec![t("Figure A1")]);

        // Test Fig with alphanumeric label
        let result = internal_link(&mut "Fig D3")?;
        assert_eq!(result.target, "#fig-d3");
        assert_eq!(result.content, vec![t("Fig D3")]);

        // Test suffix stripping for figure labels
        let result = internal_link(&mut "Figure S1A")?;
        assert_eq!(result.target, "#fig-s1");
        assert_eq!(result.content, vec![t("Figure S1A")]);

        // Test suffix stripping for table labels with complex structure
        let result = internal_link(&mut "Table Two-A")?;
        assert_eq!(result.target, "#tab-two");
        assert_eq!(result.content, vec![t("Table Two-A")]);

        // Test Appendix
        let result = internal_link(&mut "Appendix A")?;
        assert_eq!(result.target, "#app-a");
        assert_eq!(result.content, vec![t("Appendix A")]);

        // Test App. abbreviation
        let result = internal_link(&mut "App. B")?;
        assert_eq!(result.target, "#app-b");
        assert_eq!(result.content, vec![t("App. B")]);

        // Test Equation
        let result = internal_link(&mut "Equation 1")?;
        assert_eq!(result.target, "#eqn-1");
        assert_eq!(result.content, vec![t("Equation 1")]);

        // Test Eqn abbreviation
        let result = internal_link(&mut "Eqn 5")?;
        assert_eq!(result.target, "#eqn-5");
        assert_eq!(result.content, vec![t("Eqn 5")]);

        // Test Eqn. abbreviation
        let result = internal_link(&mut "Eqn. 3")?;
        assert_eq!(result.target, "#eqn-3");
        assert_eq!(result.content, vec![t("Eqn. 3")]);

        // Test figure with letter suffix - should strip suffix from target
        let result = internal_link(&mut "Figure 1B")?;
        assert_eq!(result.target, "#fig-1");
        assert_eq!(result.content, vec![t("Figure 1B")]);

        // Test table with letter suffix - should strip suffix from target
        let result = internal_link(&mut "Table 5A")?;
        assert_eq!(result.target, "#tab-5");
        assert_eq!(result.content, vec![t("Table 5A")]);

        // Test figure with multiple letter suffix
        let result = internal_link(&mut "Figure 10AB")?;
        assert_eq!(result.target, "#fig-10");
        assert_eq!(result.content, vec![t("Figure 10AB")]);

        // Test appendix should NOT strip suffix (only figs/tables)
        let result = internal_link(&mut "Appendix 1B")?;
        assert_eq!(result.target, "#app-1b");
        assert_eq!(result.content, vec![t("Appendix 1B")]);

        // Test equation should NOT strip suffix (only figs/tables)
        let result = internal_link(&mut "Equation 2A")?;
        assert_eq!(result.target, "#eqn-2a");
        assert_eq!(result.content, vec![t("Equation 2A")]);

        Ok(())
    }

    #[test]
    fn test_url_link() -> ParserResult<()> {
        // Test HTTPS URL
        let result = url_link(&mut "https://example.com")?;
        assert_eq!(result.target, "https://example.com");
        assert_eq!(result.content, vec![t("https://example.com")]);

        // Test HTTP URL
        let result = url_link(&mut "http://example.org")?;
        assert_eq!(result.target, "http://example.org");
        assert_eq!(result.content, vec![t("http://example.org")]);

        // Test HTTPS URL with path
        let result = url_link(&mut "https://github.com/owner/repo")?;
        assert_eq!(result.target, "https://github.com/owner/repo");
        assert_eq!(result.content, vec![t("https://github.com/owner/repo")]);

        // Test HTTP URL with query parameters
        let result = url_link(&mut "http://example.com/search?q=test&page=1")?;
        assert_eq!(result.target, "http://example.com/search?q=test&page=1");
        assert_eq!(
            result.content,
            vec![t("http://example.com/search?q=test&page=1")]
        );

        // Test case insensitive protocol
        let result = url_link(&mut "HTTPS://EXAMPLE.COM")?;
        assert_eq!(result.target, "HTTPS://EXAMPLE.COM");
        assert_eq!(result.content, vec![t("HTTPS://EXAMPLE.COM")]);

        // Test URL with trailing period (sentence punctuation)
        let result = url_link(&mut "https://example.com.")?;
        assert_eq!(result.target, "https://example.com");
        assert_eq!(result.content, vec![t("https://example.com")]);

        // Test URL with trailing comma
        let result = url_link(&mut "https://github.com/owner/repo,")?;
        assert_eq!(result.target, "https://github.com/owner/repo");
        assert_eq!(result.content, vec![t("https://github.com/owner/repo")]);

        // Test URL with multiple trailing punctuation
        let result = url_link(&mut "http://example.org/path?query=1!?.")?;
        assert_eq!(result.target, "http://example.org/path?query=1");
        assert_eq!(result.content, vec![t("http://example.org/path?query=1")]);

        // Test URL with internal dots (should preserve them)
        let result = url_link(&mut "https://api.example.com/v1/data.json")?;
        assert_eq!(result.target, "https://api.example.com/v1/data.json");
        assert_eq!(
            result.content,
            vec![t("https://api.example.com/v1/data.json")]
        );

        Ok(())
    }

    #[test]
    fn test_text_with_links() -> ParserResult<()> {
        // Test text with single link
        let result = text_with_links(&mut "See Figure 1 for details")?;
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], t("See "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#fig-1");
            assert_eq!(link.content, vec![t("Figure 1")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(" for details"));

        // Test text with multiple links
        let result = text_with_links(&mut "Figure 1 and Table 2 show the results")?;
        assert_eq!(result.len(), 4);
        if let Inline::Link(link) = &result[0] {
            assert_eq!(link.target, "#fig-1");
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[1], t(" and "));
        if let Inline::Link(link) = &result[2] {
            assert_eq!(link.target, "#tab-2");
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[3], t(" show the results"));

        // Test text with no links
        let result = text_with_links(&mut "This is plain text with no links")?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], t("This is plain text with no links"));

        // Test text with Fig. abbreviation
        let result = text_with_links(&mut "Reference Fig. 3 here")?;
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], t("Reference "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#fig-3");
            assert_eq!(link.content, vec![t("Fig. 3")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(" here"));

        // Test text with appendix link
        let result = text_with_links(&mut "See Appendix A for more details")?;
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], t("See "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#app-a");
            assert_eq!(link.content, vec![t("Appendix A")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(" for more details"));

        // Test text with equation link
        let result = text_with_links(&mut "As shown in Equation 2, the result is clear")?;
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], t("As shown in "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#eqn-2");
            assert_eq!(link.content, vec![t("Equation 2")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(", the result is clear"));

        // Test mixed link types
        let result = text_with_links(&mut "See Figure 1, Table 2, App. A and Eqn. 5")?;
        assert_eq!(result.len(), 8);
        assert_eq!(result[0], t("See "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#fig-1");
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(", "));
        if let Inline::Link(link) = &result[3] {
            assert_eq!(link.target, "#tab-2");
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[4], t(", "));
        if let Inline::Link(link) = &result[5] {
            assert_eq!(link.target, "#app-a");
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[6], t(" and "));
        if let Inline::Link(link) = &result[7] {
            assert_eq!(link.target, "#eqn-5");
        } else {
            panic!("Expected Link");
        }

        // Test links within parentheses
        let result = text_with_links(&mut "The results show clear trends (Figure 1, Table 2)")?;
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], t("The results show clear trends ("));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#fig-1");
            assert_eq!(link.content, vec![t("Figure 1")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(", "));
        if let Inline::Link(link) = &result[3] {
            assert_eq!(link.target, "#tab-2");
            assert_eq!(link.content, vec![t("Table 2")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[4], t(")"));

        // Test text with HTTP URL
        let result = text_with_links(&mut "Visit https://example.com for more info")?;
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], t("Visit "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "https://example.com");
            assert_eq!(link.content, vec![t("https://example.com")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(" for more info"));

        // Test mixed internal and external links
        let result = text_with_links(&mut "See Figure 1 and https://github.com/example for code")?;
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], t("See "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#fig-1");
            assert_eq!(link.content, vec![t("Figure 1")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(" and "));
        if let Inline::Link(link) = &result[3] {
            assert_eq!(link.target, "https://github.com/example");
            assert_eq!(link.content, vec![t("https://github.com/example")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[4], t(" for code"));

        // Test URL in parentheses
        let result = text_with_links(&mut "Check the docs (https://docs.example.com) first")?;
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], t("Check the docs ("));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "https://docs.example.com");
            assert_eq!(link.content, vec![t("https://docs.example.com")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(") first"));

        // Test URL with trailing punctuation in sentence
        let result = text_with_links(&mut "Visit https://example.com. It has great docs!")?;
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], t("Visit "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "https://example.com");
            assert_eq!(link.content, vec![t("https://example.com")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(". It has great docs!"));

        // Test URL in comma-separated list
        let result =
            text_with_links(&mut "Check https://github.com, https://gitlab.com, and others")?;
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], t("Check "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "https://github.com");
            assert_eq!(link.content, vec![t("https://github.com")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(", "));
        if let Inline::Link(link) = &result[3] {
            assert_eq!(link.target, "https://gitlab.com");
            assert_eq!(link.content, vec![t("https://gitlab.com")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[4], t(", and others"));

        // Test suffix stripping in context
        let result = text_with_links(&mut "Refer to Figure S1A and Table Two-B for analysis")?;
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], t("Refer to "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#fig-s1");
            assert_eq!(link.content, vec![t("Figure S1A")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(" and "));
        if let Inline::Link(link) = &result[3] {
            assert_eq!(link.target, "#tab-two");
            assert_eq!(link.content, vec![t("Table Two-B")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[4], t(" for analysis"));

        // Test more suffix stripping examples in text
        let result = text_with_links(&mut "See Figure 1B, Figure 10AB, and Table 3C")?;
        assert_eq!(result.len(), 6);
        // Figure 1B -> #fig-1
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#fig-1");
            assert_eq!(link.content, vec![t("Figure 1B")]);
        } else {
            panic!("Expected Link");
        }
        // Figure 10AB -> #fig-10
        if let Inline::Link(link) = &result[3] {
            assert_eq!(link.target, "#fig-10");
            assert_eq!(link.content, vec![t("Figure 10AB")]);
        } else {
            panic!("Expected Link");
        }
        // Table 3C -> #tab-3
        if let Inline::Link(link) = &result[5] {
            assert_eq!(link.target, "#tab-3");
            assert_eq!(link.content, vec![t("Table 3C")]);
        } else {
            panic!("Expected Link");
        }

        Ok(())
    }

    #[test]
    fn test_is_valid_reference_label() {
        // Valid labels with digits
        assert!(is_valid_reference_label("1"));
        assert!(is_valid_reference_label("10"));
        assert!(is_valid_reference_label("1A"));
        assert!(is_valid_reference_label("1B"));
        assert!(is_valid_reference_label("10AB"));
        assert!(is_valid_reference_label("S1"));
        assert!(is_valid_reference_label("S1A"));
        assert!(is_valid_reference_label("Two-1"));
        assert!(is_valid_reference_label("A1"));
        assert!(is_valid_reference_label("123ABC")); // 3 letters max
        assert!(is_valid_reference_label("Two-A")); // Word number with suffix
        assert!(is_valid_reference_label("three-BC")); // Word number with suffix

        // Invalid labels without digits (except single letters and word numbers)
        assert!(!is_valid_reference_label("below"));
        assert!(!is_valid_reference_label("above"));
        assert!(is_valid_reference_label("A")); // Single letters are valid now
        assert!(!is_valid_reference_label("ABC")); // Multiple letters without digits are not
        assert!(!is_valid_reference_label("text"));

        // Invalid labels with false positive patterns
        assert!(!is_valid_reference_label("below"));
        assert!(!is_valid_reference_label("above"));
        assert!(!is_valid_reference_label("describes"));
        assert!(!is_valid_reference_label("shows"));
        assert!(!is_valid_reference_label("illustrates"));

        // Invalid labels with too many trailing letters
        assert!(!is_valid_reference_label("1ABCD")); // More than 3 letters
        assert!(!is_valid_reference_label("10ABCDE")); // More than 3 letters

        // Edge cases
        assert!(is_valid_reference_label("1-A"));
        assert!(is_valid_reference_label("1_B"));
        assert!(!is_valid_reference_label("1ABCD2")); // Letters followed by another digit
    }

    #[test]
    fn test_false_positive_rejection() -> ParserResult<()> {
        // These should NOT parse as links (false positives)
        assert!(internal_link(&mut "Figure below").is_err());
        assert!(internal_link(&mut "Table above").is_err());
        assert!(internal_link(&mut "Figure describes").is_err());
        assert!(internal_link(&mut "Table shows").is_err());
        assert!(internal_link(&mut "Figure illustrates").is_err());
        assert!(internal_link(&mut "Appendix below").is_err());
        assert!(internal_link(&mut "Equation above").is_err());

        // Labels without digits should be rejected (except single letters)
        assert!(internal_link(&mut "Figure A").is_ok()); // Single letters are now valid
        assert!(internal_link(&mut "Table ABC").is_err()); // Multiple letters still rejected
        assert!(internal_link(&mut "Appendix XYZ").is_err());

        // Labels with too many trailing letters should be rejected
        assert!(internal_link(&mut "Figure 1ABCD").is_err());
        assert!(internal_link(&mut "Table 10ABCDE").is_err());

        Ok(())
    }

    #[test]
    fn test_text_with_false_positives() -> ParserResult<()> {
        // Text that should NOT have links parsed out
        let result = text_with_links(&mut "This table describes the methodology")?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], t("This table describes the methodology"));

        let result = text_with_links(&mut "See figure below for details")?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], t("See figure below for details"));

        let result = text_with_links(&mut "The appendix above contains more info")?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], t("The appendix above contains more info"));

        // Mixed case - use a multi-letter example that should still be rejected
        let result = text_with_links(&mut "Figure ABC shows the results")?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], t("Figure ABC shows the results"));

        // But valid references should still work
        let result = text_with_links(&mut "Figure 1 shows the results")?;
        assert_eq!(result.len(), 2); // Should be 2: [Link, Text] since it starts with the link
        if let Inline::Link(link) = &result[0] {
            assert_eq!(link.target, "#fig-1");
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[1], t(" shows the results"));

        Ok(())
    }
}
