use winnow::{
    Parser, Result,
    ascii::{multispace0, multispace1},
    combinator::{alt, separated, terminated},
};

use codec::{
    common::itertools::Itertools,
    schema::{Author, Citation, CitationOptions, Person, shortcuts::t},
};

use crate::decode::{
    parts::{
        authors::{etal, names},
        date::year_az,
    },
    reference::generate_id,
};

/// Parse a single author-year citation string
///
/// Parses strings like "Smith 1990", "Smith & Jones, 1990" as found within
/// parenthetical citations.
pub fn author_year(input: &mut &str) -> Result<Citation> {
    (
        alt((
            // First author et al
            terminated(
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
        )),
        alt(((multispace0, ",", multispace0).take(), multispace1)),
        year_az,
    )
        .map(|(authors, _, date_suffix)| {
            let authors = authors
                .into_iter()
                .map(|family_names| {
                    Author::Person(Person {
                        family_names: Some(family_names),
                        ..Default::default()
                    })
                })
                .collect_vec();

            let target = generate_id(&authors, &Some(date_suffix));

            Citation {
                target,
                ..Default::default()
            }
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_dev::pretty_assertions::assert_eq;

    #[test]
    fn test_names() -> Result<()> {
        let result = names(&mut "Smith")?;
        assert_eq!(result, vec!["Smith"]);

        let result = names(&mut "Smith Jones")?;
        assert_eq!(result, vec!["Smith", "Jones"]);

        Ok(())
    }

    #[test]
    fn test_author_year_citation() -> Result<()> {
        // Single author variations
        let citation = author_year(&mut "Smith 1990")?;
        assert_eq!(citation.target, "smith-1990");

        let citation = author_year(&mut "Smith, 1990")?;
        assert_eq!(citation.target, "smith-1990");

        // Single author with year suffix
        let citation = author_year(&mut "Jones 2023a")?;
        assert_eq!(citation.target, "jones-2023a");

        let citation = author_year(&mut "Jones, 2023z")?;
        assert_eq!(citation.target, "jones-2023z");

        // Multi-part family name
        let citation = author_year(&mut "Van Der Berg 2017")?;
        assert_eq!(citation.target, "van-der-berg-2017");

        let citation = author_year(&mut "Van Der Berg, 2017")?;
        assert_eq!(citation.target, "van-der-berg-2017");

        // Hyphenated family name
        let citation = author_year(&mut "Smith-Jones 2016")?;
        assert_eq!(citation.target, "smith-jones-2016");

        let citation = author_year(&mut "O'Connor, 2015")?;
        assert_eq!(citation.target, "o-connor-2015");

        // Two authors with ampersand
        let citation = author_year(&mut "Smith & Jones 1995")?;
        assert_eq!(citation.target, "smith-and-jones-1995");

        let citation = author_year(&mut "Smith&Jones 1995")?;
        assert_eq!(citation.target, "smith-and-jones-1995");

        let citation = author_year(&mut "Smith & Jones, 1995")?;
        assert_eq!(citation.target, "smith-and-jones-1995");

        // Two authors with "and"
        let citation = author_year(&mut "Taylor and Wilson 2020")?;
        assert_eq!(citation.target, "taylor-and-wilson-2020");

        let citation = author_year(&mut "Taylor and Wilson, 2020")?;
        assert_eq!(citation.target, "taylor-and-wilson-2020");

        // Et al variations
        let citation = author_year(&mut "Johnson et al. 2019")?;
        assert_eq!(citation.target, "johnson-et-al-2019");

        let citation = author_year(&mut "Johnson et al., 2019")?;
        assert_eq!(citation.target, "johnson-et-al-2019");

        let citation = author_year(&mut "Garcia et al 2018")?;
        assert_eq!(citation.target, "garcia-et-al-2018");

        let citation = author_year(&mut "Garcia, et al., 2018")?;
        assert_eq!(citation.target, "garcia-et-al-2018");

        // Year range variations
        let citation = author_year(&mut "Brown 1200")?;
        assert_eq!(citation.target, "brown-1200");

        let citation = author_year(&mut "Miller 2050")?;
        assert_eq!(citation.target, "miller-2050");

        // Complex multi-part names
        let citation = author_year(&mut "Von Der Leyen & Garcia-Martinez 2021")?;
        assert_eq!(citation.target, "von-der-leyen-and-garcia-martinez-2021");

        // Non-matches - invalid years
        assert!(author_year(&mut "Smith 1199").is_err()); // Year too early
        assert!(author_year(&mut "Smith 2051").is_err()); // Year too late
        assert!(author_year(&mut "Smith 999").is_err()); // Year too short

        // Non-matches - missing components
        assert!(author_year(&mut "Smith").is_err()); // No year
        assert!(author_year(&mut "1990").is_err()); // No author
        assert!(author_year(&mut "").is_err()); // Empty string

        // Valid multi-word family name (actually matches as single author)
        let citation = author_year(&mut "Smith Jones 1990")?;
        assert_eq!(citation.target, "smith-jones-1990");

        // Non-matches - invalid formats
        assert!(author_year(&mut "Smith & 1990").is_err()); // Missing second author
        assert!(author_year(&mut "& Jones 1990").is_err()); // Missing first author
        assert!(author_year(&mut "Smith et 1990").is_err()); // Incomplete "et al"
        assert!(author_year(&mut "Smith al 1990").is_err()); // Missing "et"

        // Non-matches - invalid characters
        assert!(author_year(&mut "Sm1th 1990").is_err()); // Numbers in name
        assert!(author_year(&mut "Smith@ 1990").is_err()); // Special characters in name
        assert!(author_year(&mut "Smith 199a").is_err()); // Letters in year (except suffix)

        Ok(())
    }
}
