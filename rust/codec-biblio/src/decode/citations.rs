use winnow::{
    Parser, Result,
    ascii::{multispace0, multispace1},
    combinator::{alt, delimited, separated, terminated},
};

use codec::{
    common::itertools::Itertools,
    schema::{Author, Citation, CitationGroup, CitationMode, Inline, Person},
};

use crate::decode::{
    parts::{
        authors::{etal, names},
        date::year_az,
    },
    reference::generate_id,
};

/// Parse an author-year citation (either parenthetical or
/// narrative) or citation group.
fn author_year(input: &mut &str) -> Result<Inline> {
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
        .map(|(authors, _, date_suffix)| Citation {
            target: generate_id(&authors, &Some(date_suffix)),
            citation_mode: Some(CitationMode::Narrative),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse a single author-year citation string
///
/// Parses strings like "Smith 1990", "Smith & Jones, 1990a" as found within
/// parenthetical citations such as "(Smith 1990)".
fn author_year_item(input: &mut &str) -> Result<Citation> {
    (
        authors,
        alt(((multispace0, ",", multispace0).take(), multispace1)),
        year_az,
    )
        .map(|(authors, _, date_suffix)| Citation {
            target: generate_id(&authors, &Some(date_suffix)),
            ..Default::default()
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use common_dev::pretty_assertions::assert_eq;

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
        assert!(author_year_item(&mut "& Jones 1990").is_err()); // Missing first author
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
}
