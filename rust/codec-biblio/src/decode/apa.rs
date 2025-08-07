//! Parsers that parse Stencila [`Reference`] nodes from strings in APA reference list format
//!
//! This module provides parsers for extracting bibliographic information from APA
//! (American Psychological Association) style reference citations. The parsers handle
//! the standard components of APA references including authors, publication dates,
//! titles, journal information, volume/issue numbers, page ranges, and DOIs.
//!
//! # Supported Reference Types
//!
//! Currently supports:
//!
//! - **Journal Articles**: Author, A. B. (Year). Title of article. *Journal Name*, Volume(Issue), pages. DOI
//!
//! # Examples
//!
//! ```text
//! Smith, J. A., & Jones, B. C. (2023). Research methodology in psychology.
//! Journal of Applied Psychology, 108(3), 45-62. https://doi.org/10.1037/example
//! ```

use winnow::{
    Parser, Result,
    ascii::{digit1, multispace0},
    combinator::{alt, delimited, opt, preceded},
    token::take_until,
};

use codec::schema::{CreativeWorkType, IntegerOrString, Reference, shortcuts::t};

use crate::decode::{authors::authors, date::year, doi::doi, pages::pages};

/// Parse a Stencila [`Reference`] from an APA reference list item
///
/// This is the main entry point for parsing APA-style references. It attempts to
/// identify the type of reference (currently only articles) and parse accordingly.
///
/// Currently supported reference types:
///
/// - Journal articles
///
/// Future work may include books, book chapters, conference papers, etc.
pub fn apa(input: &mut &str) -> Result<Reference> {
    alt((article,)).parse_next(input)
}

/// Parse an APA journal article reference
///
/// Parses APA-style journal article references with the following expected format:
///
/// ```text
/// Author, A. B., & Author, C. D. (Year). Title of article. Journal Name, Volume(Issue), pages. DOI
/// ```
fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse one or more authors (persons or organizations)
        // Handles leading whitespace before the first author
        preceded(multispace0, authors),
        // Date: Parse year in parentheses format "(YYYY)"
        // Allows optional whitespace and trailing period
        delimited(
            (multispace0, "(", multispace0),
            year,
            (multispace0, ")", opt((multispace0, "."))),
        ),
        // Title: Parse article title ending with a period
        // Captures everything up to the first period
        delimited(multispace0, take_until(1.., '.'), "."),
        // Journal: Parse journal name ending with a comma
        // Captures everything up to the first comma
        delimited(multispace0, take_until(1.., ','), ","),
        // Volume and Issue: Parse volume number with optional issue in parentheses
        // Format: Volume or Volume(Issue)
        // Volume is required (digits), issue is optional
        delimited(
            multispace0,
            (
                digit1, // Volume number (required)
                opt(delimited(
                    (multispace0, "(", multispace0),
                    digit1, // Issue number (optional)
                    (multispace0, ")"),
                )),
            ),
            multispace0,
        ),
        // Pages: Optional page range or single page
        // Can end with period or whitespace
        opt(delimited(multispace0, pages, alt((".", multispace0)))),
        // DOI: Optional Digital Object Identifier
        // Can be a URL, prefixed, or bare DOI format
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
        // Canonical example with all components
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

        // Without issue number
        let reference = apa(&mut "Smith, J. (2020). Research methods. Science Journal, 15 45-60.")?;
        assert!(reference.is_part_of.unwrap().issue_number.is_none());

        // Without pages
        let reference = apa(&mut "Jones, A. (2021). New findings. Nature, 500(1).")?;
        assert!(reference.page_start.is_none());
        assert!(reference.page_end.is_none());

        // Without DOI
        let reference = apa(&mut "Brown, K. (2019). Analysis. Medical Journal, 25(3) 10-20.")?;
        assert!(reference.doi.is_none());

        // With extra whitespace
        let reference = apa(
            &mut "  Wilson, M.   (  2022  ) .   Title here  .   Journal Name  ,   12  (  4  )   100-110  .",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.authors.is_some());

        // With trailing period after year
        let reference =
            apa(&mut "Davis, R. (2018). Study results. Research Quarterly, 8(2) 5-15.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));

        // Single author
        let reference = apa(&mut "Taylor, S. (2023). Solo work. Solo Journal, 1(1) 1-5.")?;
        assert_eq!(reference.authors.unwrap().len(), 1);

        // Organization as author
        let reference = apa(
            &mut "World Health Organization (2020). Global report. Health Affairs, 10(5) 25-50.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));

        // Bare DOI
        let reference =
            apa(&mut "Clark, P. (2021). Findings. Tech Review, 3(1) 15-25. 10.1234/example")?;
        assert!(reference.doi.is_some());

        // Prefixed DOI
        let reference = apa(
            &mut "Miller, L. (2019). Innovation. Future Studies, 7(2) 30-45. doi:10.5678/test",
        )?;
        assert!(reference.doi.is_some());

        Ok(())
    }
}
