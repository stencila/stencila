//! Parsers that parse a Stencila [`Reference`] from a string in APA reference list format

use winnow::{
    Parser, Result,
    ascii::{digit1, multispace0},
    combinator::{alt, delimited, opt, preceded},
    token::take_until,
};

use codec::schema::{CreativeWorkType, IntegerOrString, Reference, shortcuts::t};

use crate::decode::{authors::authors, date::year, doi::doi, pages::pages};

/// Parse a Stencila [`Reference`] from an APA reference list item
pub fn apa(input: &mut &str) -> Result<Reference> {
    alt((article,)).parse_next(input)
}

/// Parse an APA [`CreativeWorkType::Article`]
fn article(input: &mut &str) -> Result<Reference> {
    (
        // authors
        preceded(multispace0, authors),
        // date
        delimited(
            (multispace0, "(", multispace0),
            year,
            (multispace0, ")", opt((multispace0, "."))),
        ),
        // title
        delimited(multispace0, take_until(1.., '.'), "."),
        // journal
        delimited(multispace0, take_until(1.., ','), ","),
        // volume and issue
        delimited(
            multispace0,
            (
                digit1,
                opt(delimited(
                    (multispace0, "(", multispace0),
                    digit1,
                    (multispace0, ")"),
                )),
            ),
            multispace0,
        ),
        // pages
        opt(delimited(multispace0, pages, alt((".", multispace0)))),
        // doi
        opt(delimited(multispace0, doi, alt((".", multispace0)))),
    )
        .map(
            |(authors, date, title, journal, (volume, issue), pages, doi)| Reference {
                work_type: Some(CreativeWorkType::Article),
                authors: Some(authors),
                date: Some(date),
                title: Some(vec![t(title)]),
                is_part_of: Some(Box::new(Reference {
                    title: Some(vec![t(journal)]),
                    volume_number: Some(IntegerOrString::from(volume)),
                    issue_number: issue.map(IntegerOrString::from),
                    ..Default::default()
                })),
                doi: doi.map(String::from),
                ..pages.unwrap_or_default()
            },
        )
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use codec_text_trait::to_text;

    use super::*;

    // These tests call the top level `apa` parser to test for discrimination
    // between different work types.
    //
    // Avoid temptation to assert parsed details of works, instead relying on
    // the unit test for sub-parsers in other modules for that, where they exist.

    #[test]
    fn test_article() -> Result<()> {
        let reference = apa(
            &mut "Author, A. B., & Author, C. D. (1999). Title of article. Title of Journal, 1(2) 34-56. https://doi.org/xyz",
        )?;

        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.authors.is_some());
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Title of article".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .and_then(|journal| journal.title)
                .map(|title| to_text(&title)),
            Some("Title of Journal".to_string())
        );
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());
        assert!(reference.doi.is_some());

        Ok(())
    }
}
