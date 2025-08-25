use winnow::{
    Parser, Result as ParserResult,
    ascii::{Caseless, multispace0, multispace1},
    combinator::{alt, not, preceded, repeat},
    token::{any, take_while},
};

use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, NodeType,
    common::{async_trait::async_trait, eyre::Result, inflector::Inflector},
    format::Format,
    schema::{Inline, Link, Node, Paragraph, shortcuts::t},
    status::Status,
};

/// A codec for links within a document
///
/// Parses both internal links like "Figure 1", "Table 2", "Appendix A", "Equation 3"
/// and external HTTP/HTTPS URLs from text.
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
/// Identifies both internal links (Figure 1, Table 2, etc.) and external HTTP/HTTPS URLs
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
        .map(|(label_type, label): (&str, &str)| {
            let id = label.to_kebab_case();

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

        // Test kebab case conversion for complex labels
        let result = internal_link(&mut "Figure S1A")?;
        assert_eq!(result.target, "#fig-s1a");
        assert_eq!(result.content, vec![t("Figure S1A")]);

        // Test kebab case conversion with spaces/hyphens
        let result = internal_link(&mut "Table Two-A")?;
        assert_eq!(result.target, "#tab-two-a");
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

        // Test kebab case conversion in context
        let result = text_with_links(&mut "Refer to Figure S1A and Table Two-B for analysis")?;
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], t("Refer to "));
        if let Inline::Link(link) = &result[1] {
            assert_eq!(link.target, "#fig-s1a");
            assert_eq!(link.content, vec![t("Figure S1A")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[2], t(" and "));
        if let Inline::Link(link) = &result[3] {
            assert_eq!(link.target, "#tab-two-b");
            assert_eq!(link.content, vec![t("Table Two-B")]);
        } else {
            panic!("Expected Link");
        }
        assert_eq!(result[4], t(" for analysis"));

        Ok(())
    }
}
