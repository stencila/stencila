use itertools::Itertools;
use winnow::{
    Parser, Result,
    ascii::{digit1, multispace0, multispace1},
    combinator::{alt, delimited, not, opt, preceded, repeat, separated, terminated},
    token::{any, take_while},
};

use codec::schema::{
    Author, Citation, CitationGroup, CitationMode, CitationOptions, Inline, Person, shortcuts::t,
};

use crate::decode::{
    parts::{
        authors::{etal, names},
        chars::is_hyphen,
        date::year_az,
    },
    reference::generate_id,
};

/// Parse author-year narrative and parenthetic citations (e.g. Smith (1990),
/// (Smith & Jones, 1990)) within text returning a vector of inlines that are
/// either [Inline::Text], [Inline::Citation], or [Inline::CitationGroup]
///
/// This intentionally consumes substrings such as ". From" and "? See" so that
/// those capitalized words are not consumed as names in a narrative citation.
pub(crate) fn author_year_and_text(input: &mut &str) -> Result<Vec<Inline>> {
    repeat(
        1..,
        alt((
            (
                alt((".", "?", "!")),
                multispace1,
                alt(("From", "See", "Cf", "In")),
                multispace1,
            )
                .take()
                .map(|text: &str| t(text)),
            author_year,
            preceded(not(author_year), any)
                .take()
                .map(|text: &str| t(text)),
        )),
    )
    .map(fold_text_inlines)
    .parse_next(input)
}

/// Parse square bracket numeric citations (e.g. [1], [1-3], [1,2,3]) within
/// text returning a vector of inlines that are either [Inline::Text],
/// [Inline::Citation], or [Inline::CitationGroup].
pub(crate) fn bracketed_numeric_and_text(input: &mut &str) -> Result<Vec<Inline>> {
    repeat(
        1..,
        alt((
            bracketed_numeric,
            preceded(not(bracketed_numeric), any)
                .take()
                .map(|text: &str| t(text)),
        )),
    )
    .map(fold_text_inlines)
    .parse_next(input)
}

/// Parse parenthetic numeric citations (e.g. (1), (1-3), (1,2,3)) within text
/// returning a vector of inlines that are either [Inline::Text],
/// [Inline::Citation], or [Inline::CitationGroup]
pub(crate) fn parenthetic_numeric_and_text(input: &mut &str) -> Result<Vec<Inline>> {
    repeat(
        1..,
        alt((
            parenthetic_numeric,
            preceded(not(parenthetic_numeric), any)
                .take()
                .map(|text: &str| t(text)),
        )),
    )
    .map(fold_text_inlines)
    .parse_next(input)
}

/// Parse superscript numeric citations (e.g. {}^{1}, {}^{1-3}, {}^{[1-3]})
/// within text returning a vector of inlines that are either [Inline::Text],
/// [Inline::Citation], or [Inline::CitationGroup]
pub(crate) fn superscripted_numeric_and_text(input: &mut &str) -> Result<Vec<Inline>> {
    repeat(
        1..,
        alt((
            superscripted_numeric,
            preceded(not(superscripted_numeric), any)
                .take()
                .map(|text: &str| t(text)),
        )),
    )
    .map(fold_text_inlines)
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

/// Parse an author-year citation (either parenthetical or
/// narrative) or citation group.
pub(crate) fn author_year(input: &mut &str) -> Result<Inline> {
    alt((
        author_year_narrative.map(Inline::Citation),
        author_year_parenthetical,
    ))
    .parse_next(input)
}

/// Parse an author-year citation or citation group in parentheses
fn author_year_parenthetical(input: &mut &str) -> Result<Inline> {
    delimited(
        ("(", multispace0),
        separated(
            1..,
            author_year_item,
            (multispace0, alt((",", ";")), multispace0),
        )
        .map(|mut items: Vec<Citation>| {
            if items.len() == 1 {
                Inline::Citation(items.swap_remove(0))
            } else {
                Inline::CitationGroup(CitationGroup::new(items))
            }
        }),
        (multispace0, ")"),
    )
    .parse_next(input)
}

/// Parse a single author-year citation string with year in parentheses
///
/// Parses narrative author-year citations such as "Smith (1990)" and "Smith & Jones (1990a)"
fn author_year_narrative(input: &mut &str) -> Result<Citation> {
    (
        authors,
        multispace1,
        delimited(("(", multispace0), year_az, (multispace0, ")")),
    )
        .map(|(authors, _, date_suffix)| {
            let authors_string = if authors.len() > 2
                && let Some(first) = authors.first()
            {
                [&first.name(), " et al."].concat()
            } else {
                authors.iter().map(|author| author.name()).join(" and ")
            };

            let content = [
                &authors_string,
                " (",
                date_suffix.0.value.as_str(),
                date_suffix.1.as_deref().unwrap_or_default(),
                ")",
            ]
            .concat();

            Citation {
                target: generate_id(&authors, &Some(date_suffix)),
                citation_mode: Some(CitationMode::Narrative),
                options: Box::new(CitationOptions {
                    content: Some(vec![t(content)]),
                    ..Default::default()
                }),
                ..Default::default()
            }
        })
        .parse_next(input)
}

/// Parse a single author-year citation string
///
/// Parses strings like "Smith 1990", "Smith & Jones, 1990a" as found within
/// parenthetical citations such as "(Smith 1990)".
///
/// Allows for optional prefix suffix.
fn author_year_item(input: &mut &str) -> Result<Citation> {
    (
        opt(terminated(author_year_item_prefix, multispace1)),
        authors,
        alt(((multispace0, ",", multispace0).take(), multispace1)),
        year_az,
        opt(preceded(
            alt(((multispace0, ",", multispace0).take(), multispace1)),
            author_year_item_suffix,
        )),
    )
        .map(
            |(citation_prefix, authors, _, date_with_suffix, citation_suffix)| {
                let authors_string = if authors.len() > 2
                    && let Some(first) = authors.first()
                {
                    [&first.name(), " et al."].concat()
                } else {
                    authors.iter().map(|author| author.name()).join(" and ")
                };

                let content = [
                    &citation_prefix
                        .as_deref()
                        .map(|prefix| [prefix, " "].concat())
                        .unwrap_or_default(),
                    &authors_string,
                    ", ",
                    date_with_suffix.0.value.as_str(),
                    date_with_suffix.1.as_deref().unwrap_or_default(),
                    &citation_suffix
                        .as_deref()
                        .map(|suffix| [" ", suffix].concat())
                        .unwrap_or_default(),
                ]
                .concat();

                Citation {
                    target: generate_id(&authors, &Some(date_with_suffix)),
                    options: Box::new(CitationOptions {
                        citation_prefix,
                        citation_suffix,
                        content: Some(vec![t(content)]),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            },
        )
        .parse_next(input)
}

/// Parse author names
fn authors(input: &mut &str) -> Result<Vec<Author>> {
    alt((
        // First author et al
        terminated(
            // Create two extra empty authors so that et-al is generated for target
            names.map(|names| vec![names, Vec::new(), Vec::new()]),
            (
                alt(((multispace0, ",", multispace0).take(), multispace1)),
                etal,
            ),
        ),
        // Two authors
        separated(
            2..=2,
            names,
            alt((
                (multispace0, "&", multispace0),
                (multispace1, "and", multispace1),
            )),
        ),
        // Single author
        names.map(|names| vec![names]),
    ))
    .map(|authors| {
        authors
            .into_iter()
            .map(|family_names| {
                Author::Person(Person {
                    family_names: Some(family_names),
                    ..Default::default()
                })
            })
            .collect_vec()
    })
    .parse_next(input)
}

/// Parse a citation prefix within a parenthetical citation
///
/// Takes all characters until the next uppercase (to not consume author) or whitespace character
fn author_year_item_prefix(input: &mut &str) -> Result<String> {
    take_while(1.., |c: char| !c.is_whitespace() && !c.is_uppercase())
        .map(String::from)
        .parse_next(input)
}

/// Parse a citation suffix within a parenthetical citation
///
/// Takes everything until the next separator between items, or the closing
/// parenthesis. Rejects suffixes that look like complete author-year citations.
fn author_year_item_suffix(input: &mut &str) -> Result<String> {
    take_while(1.., |c: char| c != ',' && c != ';' && c != ')')
        .verify(|suffix: &str| {
            let trimmed = suffix.trim();
            if trimmed.is_empty() {
                return false;
            }
            // Ensure that this is not a complete author-year item
            let mut test_input = trimmed;
            author_year_item(&mut test_input).is_err()
        })
        .map(String::from)
        .parse_next(input)
}

/// Parse square bracket citation like [1,2,3-5]
pub(crate) fn bracketed_numeric(input: &mut &str) -> Result<Inline> {
    delimited("[", citation_sequence, "]").parse_next(input)
}

/// Parse parentheses citation like (1,2,3-5)
pub(crate) fn parenthetic_numeric(input: &mut &str) -> Result<Inline> {
    delimited("(", citation_sequence, ")").parse_next(input)
}

/// Parse superscript math citation like {}^{1,2,3-5} and {}^{[1,2,3-5]}
pub(crate) fn superscripted_numeric(input: &mut &str) -> Result<Inline> {
    delimited(
        ("{", multispace0, "}", "^", "{", multispace0),
        alt((
            citation_sequence,
            delimited(("[", multispace0), citation_sequence, (multispace0, "]")),
        )),
        (multispace0, "}"),
    )
    .parse_next(input)
}

/// Parse a sequence of citation numbers with commas and dashes
fn citation_sequence(input: &mut &str) -> Result<Inline> {
    separated(
        1..,
        alt((citation_range, citation_number)),
        (multispace0, ",", multispace0),
    )
    .verify(|items: &Vec<CitationItem>| !items.is_empty())
    .map(|items| {
        let citations = expand_citation_items(items);
        if let Some(citation) = citations.first()
            && citations.len() == 1
        {
            Inline::Citation(citation.clone())
        } else {
            Inline::CitationGroup(CitationGroup::new(citations))
        }
    })
    .parse_next(input)
}

#[derive(Debug, Clone)]
enum CitationItem {
    Single(u32),
    Range(u32, u32),
}

/// Parse a citation range like "1-3" or "10-15"
fn citation_range(input: &mut &str) -> Result<CitationItem> {
    (
        citation_number_value,
        (multispace0, take_while(1..3, is_hyphen), multispace0),
        citation_number_value,
    )
        .verify(|(start, _, end)| {
            *start > 0 && *start < 500 && *end > 0 && *end < 500 && start <= end
        })
        .map(|(start, _, end)| CitationItem::Range(start, end))
        .parse_next(input)
}

/// Parse a single citation number
fn citation_number(input: &mut &str) -> Result<CitationItem> {
    citation_number_value
        .verify(|num| *num > 0 && *num < 500)
        .map(CitationItem::Single)
        .parse_next(input)
}

/// Parse just the numeric value
fn citation_number_value(input: &mut &str) -> Result<u32> {
    (multispace0, digit1, multispace0)
        .map(|(_, digits, _): (_, &str, _)| digits.parse().unwrap_or(0))
        .parse_next(input)
}

/// Expand citation items into individual Citations
fn expand_citation_items(items: Vec<CitationItem>) -> Vec<Citation> {
    let mut citations = Vec::new();
    for item in items {
        match item {
            CitationItem::Single(num) => {
                citations.push(numeric_citation(num));
            }
            CitationItem::Range(start, end) => {
                citations.extend((start..=end).map(numeric_citation));
            }
        }
    }
    citations
}

/// Create a numeric citation
fn numeric_citation(num: u32) -> Citation {
    Citation {
        target: format!("ref-{num}"),
        options: Box::new(CitationOptions {
            content: Some(vec![t(num.to_string())]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Shortcut for a narrative citation with content
    fn ctn(target: &str, content: &str) -> Inline {
        Inline::Citation(Citation {
            target: target.into(),
            citation_mode: Some(CitationMode::Narrative),
            options: Box::new(CitationOptions {
                content: Some(vec![t(content)]),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    // Shortcut for a parenthetical citation with content
    fn ctp(target: &str, content: &str) -> Inline {
        Inline::Citation(Citation {
            target: target.into(),
            options: Box::new(CitationOptions {
                content: Some(vec![t(content)]),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    /// Shortcut for a [`Citation`] with content
    fn ctg<const N: usize>(citations: [(&str, &str); N]) -> Inline {
        Inline::CitationGroup(CitationGroup::new(
            citations
                .into_iter()
                .map(|(target, content)| Citation {
                    target: target.to_string(),
                    options: Box::new(CitationOptions {
                        content: Some(vec![t(content)]),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .collect(),
        ))
    }

    #[test]
    fn test_author_year_text() -> Result<()> {
        // Simple text with narrative citation
        let inlines = author_year_and_text(&mut "According to Smith (1990),..")?;
        assert_eq!(
            inlines,
            vec![
                t("According to "),
                ctn("smith-1990", "Smith (1990)"),
                t(",..")
            ]
        );

        // Simple text with parenthetical citation
        let inlines = author_year_and_text(&mut "The theory holds (Smith 1990).")?;
        assert_eq!(
            inlines,
            vec![
                t("The theory holds "),
                ctp("smith-1990", "Smith, 1990"),
                t(".")
            ]
        );

        // Simple text with citation group using semicolon separator
        let inlines = author_year_and_text(&mut "The theory holds (Smith 1990; Jones 1991).")?;
        assert_eq!(
            inlines,
            vec![
                t("The theory holds "),
                ctg([("smith-1990", "Smith, 1990"), ("jones-1991", "Jones, 1991")]),
                t(".")
            ]
        );

        // Text with multiple narrative citations
        let inlines =
            author_year_and_text(&mut "Smith (1990) argued that Jones (2000) was correct.")?;
        assert_eq!(
            inlines,
            vec![
                ctn("smith-1990", "Smith (1990)"),
                t(" argued that "),
                ctn("jones-2000", "Jones (2000)"),
                t(" was correct.")
            ]
        );

        // Narrative citations should not consume common capitalized letters at start of
        // sentence
        let inlines = author_year_and_text(&mut ". From Brown (1995)! See Smith (2010a).")?;
        assert_eq!(
            inlines,
            vec![
                t(". From "),
                ctn("brown-1995", "Brown (1995)"),
                t("! See "),
                ctn("smith-2010a", "Smith (2010a)"),
                t(".")
            ]
        );

        // Text with multiple parenthetical citations
        let inlines =
            author_year_and_text(&mut "Studies show (Brown 1995) and (Wilson 2001) findings.")?;
        assert_eq!(
            inlines,
            vec![
                t("Studies show "),
                ctp("brown-1995", "Brown, 1995"),
                t(" and "),
                ctp("wilson-2001", "Wilson, 2001"),
                t(" findings.")
            ]
        );

        // Text with mixed citation types
        let inlines = author_year_and_text(
            &mut "According to Davis (2010), multiple studies (Taylor 2015; Miller 2020) confirm this.",
        )?;
        assert_eq!(
            inlines,
            vec![
                t("According to "),
                ctn("davis-2010", "Davis (2010)"),
                t(", multiple studies "),
                ctg([
                    ("taylor-2015", "Taylor, 2015"),
                    ("miller-2020", "Miller, 2020")
                ]),
                t(" confirm this.")
            ]
        );

        // Text with no citations (plain text)
        let inlines =
            author_year_and_text(&mut "This is just regular text without any citations.")?;
        assert_eq!(
            inlines,
            vec![t("This is just regular text without any citations.")]
        );

        // Text starting with a citation
        let inlines =
            author_year_and_text(&mut "Smith (1990) was the first to discover this phenomenon.")?;
        assert_eq!(
            inlines,
            vec![
                ctn("smith-1990", "Smith (1990)"),
                t(" was the first to discover this phenomenon.")
            ]
        );

        // Text ending with a citation
        let inlines = author_year_and_text(&mut "This discovery was made by Smith (1990)")?;
        assert_eq!(
            inlines,
            vec![
                t("This discovery was made by "),
                ctn("smith-1990", "Smith (1990)")
            ]
        );

        // Text with citation containing prefixes and suffixes
        let inlines =
            author_year_and_text(&mut "Research shows (see Jones 2020, p. 15) that this is true.")?;
        assert_eq!(inlines.len(), 3);
        assert_eq!(inlines[0], t("Research shows "));
        match &inlines[1] {
            Inline::Citation(citation) => {
                assert_eq!(citation.target, "jones-2020");
                assert_eq!(citation.options.citation_prefix, Some("see".to_string()));
                assert_eq!(citation.options.citation_suffix, Some("p. 15".to_string()));
            }
            _ => panic!("Expected citation"),
        }
        assert_eq!(inlines[2], t(" that this is true."));

        // Text with adjacent text that should be folded together
        let inlines = author_year_and_text(&mut "Hello world from the author.")?;
        assert_eq!(inlines, vec![t("Hello world from the author.")]);

        // Text with special characters around citations
        let inlines =
            author_year_and_text(&mut "The study (Smith 1990) shows: important findings!")?;
        assert_eq!(
            inlines,
            vec![
                t("The study "),
                ctp("smith-1990", "Smith, 1990"),
                t(" shows: important findings!")
            ]
        );

        // Text with parentheses that are not citations
        let inlines = author_year_and_text(&mut "Some text (not a citation) and more text.")?;
        assert_eq!(
            inlines,
            vec![t("Some text (not a citation) and more text.")]
        );

        // Text with year but no valid author format
        let inlines = author_year_and_text(&mut "The year (1990) was significant.")?;
        assert_eq!(inlines, vec![t("The year (1990) was significant.")]);

        // Citation with et al
        let inlines = author_year_and_text(
            &mut "Studies by Johnson et al. (2019) show interesting results.",
        )?;
        assert_eq!(
            inlines,
            vec![
                t("Studies by "),
                ctn("johnson-et-al-2019", "Johnson et al. (2019)"),
                t(" show interesting results.")
            ]
        );

        // Citation with two authors using ampersand
        let inlines =
            author_year_and_text(&mut "The work of Smith & Jones (1995) was groundbreaking.")?;
        assert_eq!(
            inlines,
            vec![
                t("The work of "),
                ctn("smith-and-jones-1995", "Smith and Jones (1995)"),
                t(" was groundbreaking.")
            ]
        );

        // Citation with complex citation group with different separators
        let inlines = author_year_and_text(
            &mut "Multiple studies (Brown 2010; Davis, 2015; Wilson 2020) support this.",
        )?;
        assert_eq!(
            inlines,
            vec![
                t("Multiple studies "),
                ctg([
                    ("brown-2010", "Brown, 2010"),
                    ("davis-2015", "Davis, 2015"),
                    ("wilson-2020", "Wilson, 2020")
                ]),
                t(" support this.")
            ]
        );

        Ok(())
    }

    #[test]
    fn test_author_year() -> Result<()> {
        // Single narrative citation
        let inline = author_year(&mut "Smith (1990)")?;
        match inline {
            Inline::Citation(Citation {
                target,
                citation_mode: Some(CitationMode::Narrative),
                ..
            }) => assert_eq!(target, "smith-1990"),
            _ => unreachable!("expected single narrative citation"),
        }

        // Single parenthetical citation
        let inline = author_year(&mut "(Smith 1990)")?;
        match inline {
            Inline::Citation(Citation {
                target,
                citation_mode: None,
                ..
            }) => {
                assert_eq!(target, "smith-1990")
            }
            _ => unreachable!("expected single parenthetical citation"),
        }

        // Citation group with two citations
        let inline = author_year(&mut "(Smith 1990; Jones 1995)")?;
        match inline {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 2);
                assert_eq!(group.items[0].target, "smith-1990");
                assert_eq!(group.items[1].target, "jones-1995");
            }
            _ => unreachable!("expected citation group"),
        }

        // Citation group with comma separator
        let inline = author_year(&mut "(Brown 2020, Wilson 2021)")?;
        match inline {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 2);
                assert_eq!(group.items[0].target, "brown-2020");
                assert_eq!(group.items[1].target, "wilson-2021");
            }
            _ => unreachable!("expected citation group"),
        }

        // Citation group with mixed separators (semicolons and commas)
        let inline = author_year(&mut "(Smith 1990; Brown, 2020; Wilson 2021)")?;
        match inline {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 3);
                assert_eq!(group.items[0].target, "smith-1990");
                assert_eq!(group.items[1].target, "brown-2020");
                assert_eq!(group.items[2].target, "wilson-2021");
            }
            _ => unreachable!("expected citation group"),
        }

        // Citation with page suffix
        let inline = author_year(&mut "(Smith 1990, p. 15)")?;
        match inline {
            Inline::Citation(Citation {
                target, options, ..
            }) => {
                assert_eq!(target, "smith-1990");
                assert_eq!(options.citation_suffix, Some("p. 15".to_string()));
            }
            _ => unreachable!("expected single citation"),
        }

        // Citation group with page suffixes
        let inline = author_year(&mut "(Smith 1990, pp. 1-5; Jones 2020, p. 10)")?;
        match inline {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 2);
                assert_eq!(group.items[0].target, "smith-1990");
                assert_eq!(
                    group.items[0].options.citation_suffix,
                    Some("pp. 1-5".to_string())
                );
                assert_eq!(group.items[1].target, "jones-2020");
                assert_eq!(
                    group.items[1].options.citation_suffix,
                    Some("p. 10".to_string())
                );
            }
            _ => unreachable!("expected citation group"),
        }

        // Citation group with complex suffixes
        let inline = author_year(&mut "(Davis 2021 Table 3.1; Miller 2022, Appendix B.2.3)")?;
        match inline {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 2);
                assert_eq!(group.items[0].target, "davis-2021");
                assert_eq!(
                    group.items[0].options.citation_suffix,
                    Some("Table 3.1".to_string())
                );
                assert_eq!(group.items[1].target, "miller-2022");
                assert_eq!(
                    group.items[1].options.citation_suffix,
                    Some("Appendix B.2.3".to_string())
                );
            }
            _ => unreachable!("expected citation group"),
        }

        // Citation group with prefixes and suffixes
        let inline =
            author_year(&mut "(see Smith 1990 p. 5; cf. Jones 2021; also Brown 2022, Table 1)")?;
        match inline {
            Inline::CitationGroup(group) => {
                assert_eq!(group.items.len(), 3);
                assert_eq!(group.items[0].target, "smith-1990");
                assert_eq!(
                    group.items[0].options.citation_prefix,
                    Some("see".to_string())
                );
                assert_eq!(
                    group.items[0].options.citation_suffix,
                    Some("p. 5".to_string())
                );
                assert_eq!(group.items[1].target, "jones-2021");
                assert_eq!(
                    group.items[1].options.citation_prefix,
                    Some("cf.".to_string())
                );
                assert_eq!(group.items[2].target, "brown-2022");
                assert_eq!(
                    group.items[2].options.citation_prefix,
                    Some("also".to_string())
                );
                assert_eq!(
                    group.items[2].options.citation_suffix,
                    Some("Table 1".to_string())
                );
            }
            _ => unreachable!("expected citation group"),
        }

        Ok(())
    }

    #[test]
    fn test_item_prefix() -> Result<()> {
        let prefix = author_year_item_prefix(&mut "see")?;
        assert_eq!(prefix, "see");

        let prefix = author_year_item_prefix(&mut "cf.")?;
        assert_eq!(prefix, "cf.");

        let prefix = author_year_item_prefix(&mut "e.g.")?;
        assert_eq!(prefix, "e.g.");

        let prefix = author_year_item_prefix(&mut "also")?;
        assert_eq!(prefix, "also");

        Ok(())
    }

    #[test]
    fn test_item_suffix() -> Result<()> {
        let suffix = author_year_item_suffix(&mut "p. 15")?;
        assert_eq!(suffix, "p. 15");

        let suffix = author_year_item_suffix(&mut "ch. 5")?;
        assert_eq!(suffix, "ch. 5");

        let suffix = author_year_item_suffix(&mut "§ 42")?;
        assert_eq!(suffix, "§ 42");

        let suffix = author_year_item_suffix(&mut "Table 3.2 & Fig. 4")?;
        assert_eq!(suffix, "Table 3.2 & Fig. 4");

        Ok(())
    }

    #[test]
    fn test_author_year_item() -> Result<()> {
        // Single author variations
        let citation = author_year_item(&mut "Smith 1990")?;
        assert_eq!(citation.target, "smith-1990");

        let citation = author_year_item(&mut "Smith, 1990")?;
        assert_eq!(citation.target, "smith-1990");

        // Single author with year suffix
        let citation = author_year_item(&mut "Jones 2023a")?;
        assert_eq!(citation.target, "jones-2023a");

        let citation = author_year_item(&mut "Jones, 2023z")?;
        assert_eq!(citation.target, "jones-2023z");

        // Multi-part family name
        let citation = author_year_item(&mut "Van Der Berg 2017")?;
        assert_eq!(citation.target, "van-der-berg-2017");

        let citation = author_year_item(&mut "Van Der Berg, 2017")?;
        assert_eq!(citation.target, "van-der-berg-2017");

        // Hyphenated family name
        let citation = author_year_item(&mut "Smith-Jones 2016")?;
        assert_eq!(citation.target, "smith-jones-2016");

        let citation = author_year_item(&mut "O'Connor, 2015")?;
        assert_eq!(citation.target, "o-connor-2015");

        // Two authors with ampersand
        let citation = author_year_item(&mut "Smith & Jones 1995")?;
        assert_eq!(citation.target, "smith-and-jones-1995");

        let citation = author_year_item(&mut "Smith&Jones 1995")?;
        assert_eq!(citation.target, "smith-and-jones-1995");

        let citation = author_year_item(&mut "Smith & Jones, 1995")?;
        assert_eq!(citation.target, "smith-and-jones-1995");

        // Two authors with "and"
        let citation = author_year_item(&mut "Taylor and Wilson 2020")?;
        assert_eq!(citation.target, "taylor-and-wilson-2020");

        let citation = author_year_item(&mut "Taylor and Wilson, 2020")?;
        assert_eq!(citation.target, "taylor-and-wilson-2020");

        // Et al variations
        let citation = author_year_item(&mut "Johnson et al. 2019")?;
        assert_eq!(citation.target, "johnson-et-al-2019");

        let citation = author_year_item(&mut "Johnson et al., 2019")?;
        assert_eq!(citation.target, "johnson-et-al-2019");

        let citation = author_year_item(&mut "Garcia et al 2018")?;
        assert_eq!(citation.target, "garcia-et-al-2018");

        let citation = author_year_item(&mut "Garcia, et al., 2018")?;
        assert_eq!(citation.target, "garcia-et-al-2018");

        // Year range variations
        let citation = author_year_item(&mut "Brown 1200")?;
        assert_eq!(citation.target, "brown-1200");

        let citation = author_year_item(&mut "Miller 2050")?;
        assert_eq!(citation.target, "miller-2050");

        // Complex multi-part names
        let citation = author_year_item(&mut "Von Der Leyen & Garcia-Martinez 2021")?;
        assert_eq!(citation.target, "von-der-leyen-and-garcia-martinez-2021");

        // Citation suffixes - page numbers
        let citation = author_year_item(&mut "Smith 1990, p. 15")?;
        assert_eq!(citation.target, "smith-1990");
        assert_eq!(citation.options.citation_suffix, Some("p. 15".to_string()));

        let citation = author_year_item(&mut "Jones 2020, pp. 22-24")?;
        assert_eq!(citation.target, "jones-2020");
        assert_eq!(
            citation.options.citation_suffix,
            Some("pp. 22-24".to_string())
        );

        let citation = author_year_item(&mut "Brown 2021, p15")?;
        assert_eq!(citation.target, "brown-2021");
        assert_eq!(citation.options.citation_suffix, Some("p15".to_string()));

        let citation = author_year_item(&mut "Wilson 2019, pp123-456")?;
        assert_eq!(citation.target, "wilson-2019");
        assert_eq!(
            citation.options.citation_suffix,
            Some("pp123-456".to_string())
        );

        // Citation with year suffix and page suffix
        let citation = author_year_item(&mut "Taylor 2023a, p. 10")?;
        assert_eq!(citation.target, "taylor-2023a");
        assert_eq!(citation.options.citation_suffix, Some("p. 10".to_string()));

        // Multi-author with page suffix
        let citation = author_year_item(&mut "Smith & Jones 1995, pp. 1-5")?;
        assert_eq!(citation.target, "smith-and-jones-1995");
        assert_eq!(
            citation.options.citation_suffix,
            Some("pp. 1-5".to_string())
        );

        // Lenient suffix parsing - any text after comma
        let citation = author_year_item(&mut "Brown 2020, ch. 5")?;
        assert_eq!(citation.target, "brown-2020");
        assert_eq!(citation.options.citation_suffix, Some("ch. 5".to_string()));

        let citation = author_year_item(&mut "Wilson 2019, figure 2")?;
        assert_eq!(citation.target, "wilson-2019");
        assert_eq!(
            citation.options.citation_suffix,
            Some("figure 2".to_string())
        );

        let citation = author_year_item(&mut "Taylor 2021, Volume III")?;
        assert_eq!(citation.target, "taylor-2021");
        assert_eq!(
            citation.options.citation_suffix,
            Some("Volume III".to_string())
        );

        let citation = author_year_item(&mut "Garcia 2018, 00:15:30-00:20:45")?;
        assert_eq!(citation.target, "garcia-2018");
        assert_eq!(
            citation.options.citation_suffix,
            Some("00:15:30-00:20:45".to_string())
        );

        let citation = author_year_item(&mut "Miller 2022, Appendix A.2.3")?;
        assert_eq!(citation.target, "miller-2022");
        assert_eq!(
            citation.options.citation_suffix,
            Some("Appendix A.2.3".to_string())
        );

        let citation = author_year_item(&mut "Davis 2023, § 42")?;
        assert_eq!(citation.target, "davis-2023");
        assert_eq!(citation.options.citation_suffix, Some("§ 42".to_string()));

        let citation = author_year_item(&mut "Clark 2019, first half")?;
        assert_eq!(citation.target, "clark-2019");
        assert_eq!(
            citation.options.citation_suffix,
            Some("first half".to_string())
        );

        // Citation prefixes
        let citation = author_year_item(&mut "see Smith 1990")?;
        assert_eq!(citation.target, "smith-1990");
        assert_eq!(citation.options.citation_prefix, Some("see".to_string()));

        let citation = author_year_item(&mut "cf. Jones 2020")?;
        assert_eq!(citation.target, "jones-2020");
        assert_eq!(citation.options.citation_prefix, Some("cf.".to_string()));

        let citation = author_year_item(&mut "e.g. Brown 2021")?;
        assert_eq!(citation.target, "brown-2021");
        assert_eq!(citation.options.citation_prefix, Some("e.g.".to_string()));

        let citation = author_year_item(&mut "also Wilson 2019")?;
        assert_eq!(citation.target, "wilson-2019");
        assert_eq!(citation.options.citation_prefix, Some("also".to_string()));

        // Prefix and suffix combined
        let citation = author_year_item(&mut "see Taylor 2023, pp. 15-20")?;
        assert_eq!(citation.target, "taylor-2023");
        assert_eq!(citation.options.citation_prefix, Some("see".to_string()));
        assert_eq!(
            citation.options.citation_suffix,
            Some("pp. 15-20".to_string())
        );

        let citation = author_year_item(&mut "cf. Garcia 2018, Table 2")?;
        assert_eq!(citation.target, "garcia-2018");
        assert_eq!(citation.options.citation_prefix, Some("cf.".to_string()));
        assert_eq!(
            citation.options.citation_suffix,
            Some("Table 2".to_string())
        );

        // Non-matches - invalid years
        assert!(author_year_item(&mut "Smith 1199").is_err()); // Year too early
        assert!(author_year_item(&mut "Smith 2051").is_err()); // Year too late
        assert!(author_year_item(&mut "Smith 999").is_err()); // Year too short

        // Non-matches - missing components
        assert!(author_year_item(&mut "Smith").is_err()); // No year
        assert!(author_year_item(&mut "1990").is_err()); // No author
        assert!(author_year_item(&mut "").is_err()); // Empty string

        // Valid multi-word family name (actually matches as single author)
        let citation = author_year_item(&mut "Smith Jones 1990")?;
        assert_eq!(citation.target, "smith-jones-1990");

        // Non-matches - invalid formats
        assert!(author_year_item(&mut "Smith & 1990").is_err()); // Missing second author
        assert!(author_year_item(&mut "Smith et 1990").is_err()); // Incomplete "et al"

        // Non-matches - invalid characters
        assert!(author_year_item(&mut "Sm1th 1990").is_err()); // Numbers in name
        assert!(author_year_item(&mut "Smith@ 1990").is_err()); // Special characters in name
        assert!(author_year_item(&mut "Smith 199a").is_err()); // Letters in year (except suffix)

        Ok(())
    }

    #[test]
    fn test_author_year_narrative() -> Result<()> {
        // Single author
        let citation = author_year_narrative(&mut "Smith (1990)")?;
        assert_eq!(citation.target, "smith-1990");
        assert_eq!(citation.citation_mode, Some(CitationMode::Narrative));

        // Single author with year suffix
        let citation = author_year_narrative(&mut "Jones (2023a)")?;
        assert_eq!(citation.target, "jones-2023a");
        assert_eq!(citation.citation_mode, Some(CitationMode::Narrative));

        // Two authors with ampersand
        let citation = author_year_narrative(&mut "Smith & Jones (1995)")?;
        assert_eq!(citation.target, "smith-and-jones-1995");
        assert_eq!(citation.citation_mode, Some(CitationMode::Narrative));

        // Two authors with "and"
        let citation = author_year_narrative(&mut "Taylor and Wilson (2020)")?;
        assert_eq!(citation.target, "taylor-and-wilson-2020");
        assert_eq!(citation.citation_mode, Some(CitationMode::Narrative));

        // Et al
        let citation = author_year_narrative(&mut "Johnson et al. (2019)")?;
        assert_eq!(citation.target, "johnson-et-al-2019");
        assert_eq!(citation.citation_mode, Some(CitationMode::Narrative));

        // Non-matches - missing parentheses
        assert!(author_year_narrative(&mut "Smith 1990").is_err());
        assert!(author_year_narrative(&mut "Smith (1990").is_err());
        assert!(author_year_narrative(&mut "Smith 1990)").is_err());

        Ok(())
    }

    #[test]
    fn test_bracketed_numeric_and_text() -> Result<()> {
        // Square bracket citation
        let inlines = bracketed_numeric_and_text(&mut "According to [1], the study found...")?;
        assert_eq!(
            inlines,
            vec![
                t("According to "),
                ctp("ref-1", "1"),
                t(", the study found...")
            ]
        );

        // Citation group with range
        let inlines = bracketed_numeric_and_text(&mut "Studies [1-3] support this.")?;
        assert_eq!(
            inlines,
            vec![
                t("Studies "),
                ctg([("ref-1", "1"), ("ref-2", "2"), ("ref-3", "3")]),
                t(" support this.")
            ]
        );

        // Citation group with range using two hyphens
        let inlines = bracketed_numeric_and_text(&mut "As shown by [1--2,5].")?;
        assert_eq!(
            inlines,
            vec![
                t("As shown by "),
                ctg([("ref-1", "1"), ("ref-2", "2"), ("ref-5", "5")]),
                t(".")
            ]
        );

        // Should not match parentheses
        let inlines = bracketed_numeric_and_text(&mut "The equation (1) shows...")?;
        assert_eq!(inlines, vec![t("The equation (1) shows...")]);

        Ok(())
    }

    #[test]
    fn test_parenthetic_numeric_and_text() -> Result<()> {
        // Parenthetic citation
        let inlines = parenthetic_numeric_and_text(&mut "The theory holds (2).")?;
        assert_eq!(
            inlines,
            vec![t("The theory holds "), ctp("ref-2", "2"), t(".")]
        );

        // Citation group
        let inlines = parenthetic_numeric_and_text(&mut "Studies (1,3-5) confirm this.")?;
        assert_eq!(
            inlines,
            vec![
                t("Studies "),
                ctg([
                    ("ref-1", "1"),
                    ("ref-3", "3"),
                    ("ref-4", "4"),
                    ("ref-5", "5")
                ]),
                t(" confirm this.")
            ]
        );

        // Should not match square brackets
        let inlines = parenthetic_numeric_and_text(&mut "The reference [1] shows...")?;
        assert_eq!(inlines, vec![t("The reference [1] shows...")]);

        Ok(())
    }

    #[test]
    fn test_superscripted_numeric_and_text() -> Result<()> {
        // Superscript citation
        let inlines = superscripted_numeric_and_text(&mut "The study{}^{1} shows results.")?;
        assert_eq!(
            inlines,
            vec![t("The study"), ctp("ref-1", "1"), t(" shows results.")]
        );

        // Citation group
        let inlines = superscripted_numeric_and_text(&mut "Research{}^{1,3-5} confirms this.")?;
        assert_eq!(
            inlines,
            vec![
                t("Research"),
                ctg([
                    ("ref-1", "1"),
                    ("ref-3", "3"),
                    ("ref-4", "4"),
                    ("ref-5", "5")
                ]),
                t(" confirms this.")
            ]
        );

        // With inner brackets and double hyphen
        let inlines = superscripted_numeric_and_text(&mut "as shown {}^{[1,3--5]}.")?;
        assert_eq!(
            inlines,
            vec![
                t("as shown "),
                ctg([
                    ("ref-1", "1"),
                    ("ref-3", "3"),
                    ("ref-4", "4"),
                    ("ref-5", "5")
                ]),
                t("."),
            ]
        );

        // Should not match square brackets or parentheses
        let inlines =
            superscripted_numeric_and_text(&mut "The reference [1] and equation (2) show...")?;
        assert_eq!(
            inlines,
            vec![t("The reference [1] and equation (2) show...")]
        );

        Ok(())
    }
}
