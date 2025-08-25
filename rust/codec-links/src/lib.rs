use winnow::{
    Parser, Result as ParserResult,
    ascii::{Caseless, multispace0, multispace1},
    combinator::{alt, not, preceded, repeat},
    token::{any, take_while},
};

use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, NodeType,
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Inline, Link, Node, Paragraph, shortcuts::t},
    status::Status,
};

/// A codec for internal links within a document
///
/// Parses links like "Figure 1" and "(see Table 9)" from text.
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
            link.map(Inline::Link),
            preceded(not(link), any).take().map(|text: &str| t(text)),
        )),
    )
    .map(fold_text_inlines)
    .parse_next(input)
}

/// Parse a link
fn link(input: &mut &str) -> ParserResult<Link> {
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
        take_while(1.., |c: char| c.is_alphanumeric()),
    )
        .map(|(label_type, label): (&str, &str)| {
            let target = if label_type.to_lowercase().starts_with("fig") {
                ["#fig-", label].concat()
            } else if label_type.to_lowercase().starts_with("tab") {
                ["#tab-", label].concat()
            } else if label_type.to_lowercase().starts_with("app") {
                ["#app-", label].concat()
            } else if label_type.to_lowercase().starts_with("eq") {
                ["#eqn-", label].concat()
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
    fn test_link() -> ParserResult<()> {
        // Test Figure with number
        let result = link(&mut "Figure 1")?;
        assert_eq!(result.target, "#fig-1");
        assert_eq!(result.content, vec![t("Figure 1")]);

        // Test Fig. abbreviation
        let result = link(&mut "Fig. 2")?;
        assert_eq!(result.target, "#fig-2");
        assert_eq!(result.content, vec![t("Fig. 2")]);

        // Test Table
        let result = link(&mut "Table 5")?;
        assert_eq!(result.target, "#tab-5");
        assert_eq!(result.content, vec![t("Table 5")]);

        // Test case insensitive
        let result = link(&mut "figure 10")?;
        assert_eq!(result.target, "#fig-10");
        assert_eq!(result.content, vec![t("figure 10")]);

        // Test alphanumeric label
        let result = link(&mut "Figure A1")?;
        assert_eq!(result.target, "#fig-A1");
        assert_eq!(result.content, vec![t("Figure A1")]);

        // Test Appendix
        let result = link(&mut "Appendix A")?;
        assert_eq!(result.target, "#app-A");
        assert_eq!(result.content, vec![t("Appendix A")]);

        // Test App. abbreviation
        let result = link(&mut "App. B")?;
        assert_eq!(result.target, "#app-B");
        assert_eq!(result.content, vec![t("App. B")]);

        // Test Equation
        let result = link(&mut "Equation 1")?;
        assert_eq!(result.target, "#eqn-1");
        assert_eq!(result.content, vec![t("Equation 1")]);

        // Test Eqn abbreviation
        let result = link(&mut "Eqn 5")?;
        assert_eq!(result.target, "#eqn-5");
        assert_eq!(result.content, vec![t("Eqn 5")]);

        // Test Eqn. abbreviation
        let result = link(&mut "Eqn. 3")?;
        assert_eq!(result.target, "#eqn-3");
        assert_eq!(result.content, vec![t("Eqn. 3")]);

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
            assert_eq!(link.target, "#app-A");
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
            assert_eq!(link.target, "#app-A");
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

        Ok(())
    }
}
