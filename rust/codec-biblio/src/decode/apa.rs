//! Parsers that parse Stencila [`Reference`] nodes from strings in APA reference list format
//!
//! This module provides parsers for extracting bibliographic information from APA
//! (American Psychological Association) style reference citations. The parsers handle
//! the standard components of APA references including authors, publication dates,
//! titles, journal information, volume/issue numbers, page ranges, and DOIs.

use codec::schema::{Date, Inline, PropertyValueOrString};
use winnow::{
    Parser, Result,
    ascii::{digit1, multispace0, multispace1},
    combinator::{alt, delimited, not, opt, preceded, terminated},
    token::{take_until, take_while},
};

use codec::schema::{
    CreativeWorkType, IntegerOrString, Organization, PersonOrOrganization, Reference, shortcuts::t,
};

use crate::decode::{
    authors::{authors, persons},
    date::year,
    doi::doi,
    pages::pages,
};

/// Parse a Stencila [`Reference`] from an APA reference list item
///
/// This is the main entry point for parsing APA-style references. It attempts to
/// identify the type of reference and parse accordingly.
///
/// Currently supported reference types:
///
/// - Journal articles
/// - Books
/// - Book chapters
/// - Web resources
///
/// Future work may include book chapters, conference papers, etc.
pub fn apa(input: &mut &str) -> Result<Reference> {
    alt((article, chapter, web, book)).parse_next(input)
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
        apa_year,
        // Title: Parse article title ending with a period
        apa_title,
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
                title: Some(title),
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

/// Parse an APA book reference
///
/// Parses APA-style book references with the following expected format:
///
/// ```text
/// Author, A. B. (Year). Book title. Publisher. DOI
/// ```
fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse book authors
        preceded(multispace0, authors),
        // Date: Parse year in parentheses format "(YYYY)"
        apa_year,
        // Title: Parse book title ending with a period
        apa_title,
        // Publisher: Parse publisher ending with period
        delimited(multispace0, take_until(1.., '.'), "."),
        // DOI: Optional Digital Object Identifier
        opt(delimited(multispace0, doi, alt((".", multispace0)))),
    )
        // Map the parsed components into a Reference struct
        .map(|(authors, date, title, publisher, doi)| Reference {
            work_type: Some(CreativeWorkType::Book),
            authors: Some(authors),
            date: Some(date),
            title: Some(title),
            publisher: Some(PersonOrOrganization::Organization(Organization {
                name: Some(publisher.trim().to_string()),
                ..Default::default()
            })),
            doi: doi.map(String::from),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse an APA book chapter reference
///
/// Parses APA-style book chapter references with the following expected format:
///
/// ```text
/// Author, A. B. (Year). Chapter title. In Editor, E. D. (Ed.), Book title (pages). Publisher. DOI
/// ```
fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse chapter authors
        preceded(multispace0, authors),
        // Date: Parse year in parentheses
        apa_year,
        // Chapter Title: Parse chapter title ending with period
        apa_title,
        // "In" keyword with space
        delimited(multispace0, "In", multispace1),
        // Editors: before (Ed.) or (Eds.)
        // Allows for variations such as (Ed) ( Eds) ( Ed. )
        terminated(
            persons,
            (
                opt(delimited(
                    multispace0,
                    delimited(
                        ("(", multispace0),
                        (alt(("Eds", "Ed")), opt(".")),
                        (multispace0, ")"),
                    ),
                    multispace0,
                )),
                (multispace0, ",", multispace0),
            ),
        ),
        // Book Title: Parse book title before opening parenthesis
        preceded(multispace0, take_until(1.., '(')),
        // Pages: Parse page range in parentheses
        opt(delimited(
            (multispace0, "(", multispace0),
            pages,
            (multispace0, ")", opt((multispace0, "."))),
        )),
        // Publisher: Parse publisher ending with period
        opt(delimited(
            multispace0,
            take_while(1.., |c: char| c != '.'),
            opt("."),
        )),
        // DOI: Optional Digital Object Identifier
        opt(delimited(multispace0, doi, alt((".", multispace0)))),
    )
        // Map the parsed components into a Reference struct
        .map(
            |(authors, date, chapter_title, _, editors, book_title, pages, publisher, doi)| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    authors: Some(authors),
                    date: Some(date),
                    title: Some(chapter_title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(book_title.trim().to_string())]),
                        editors: Some(editors),
                        publisher: publisher.map(|publisher| {
                            PersonOrOrganization::Organization(Organization {
                                name: Some(publisher.trim().to_string()),
                                ..Default::default()
                            })
                        }),
                        ..Default::default()
                    })),
                    doi: doi.map(String::from),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an APA web resource reference
///
/// Parses APA-style web resource references with the following expected format:
///
/// ```text
/// Author, A. B. (Year). Title of webpage. Website Name. URL
/// ```
fn web(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse web authors (may be missing for some web content)
        opt(preceded(multispace0, authors)),
        // Date: Parse year in parentheses format "(YYYY)"
        apa_year,
        // Title: Parse web page title ending with a period
        apa_title,
        // Website: Parse website name ending with period
        delimited(multispace0, take_until(1.., '.'), "."),
        // URL: Capture non-DOI URLs starting with http/https
        preceded(
            multispace0,
            (
                alt(("https://", "http://")),
                // Ensure it's not a DOI URL by checking it doesn't start with doi.org, dx.doi.org, or www.doi.org
                preceded(
                    not(alt(("doi.org/", "dx.doi.org/", "www.doi.org/"))),
                    take_while(1.., |c: char| !c.is_ascii_whitespace()),
                ),
            ),
        ),
    )
        // Map the parsed components into a Reference struct
        .map(
            |(authors, date, title, website, (protocol, domain))| Reference {
                work_type: Some(CreativeWorkType::WebPage),
                authors,
                date: Some(date),
                title: Some(title),
                // Website information stored as nested Reference in is_part_of
                is_part_of: Some(Box::new(Reference {
                    title: Some(vec![t(website.trim())]),
                    ..Default::default()
                })),
                // Store full URL as identifier
                identifiers: Some(vec![PropertyValueOrString::String(format!(
                    "{}{}",
                    protocol, domain
                ))]),
                ..Default::default()
            },
        )
        .parse_next(input)
}

/// Parse year in parentheses format "(YYYY)"
///
/// Allows optional whitespace and trailing period
fn apa_year(input: &mut &str) -> Result<Date> {
    delimited(
        (multispace0, "(", multispace0),
        year,
        (multispace0, ")", opt((multispace0, "."))),
    )
    .parse_next(input)
}

/// Parse article title ending with a period
///
/// Captures everything up to the first period
fn apa_title(input: &mut &str) -> Result<Vec<Inline>> {
    delimited(multispace0, take_until(1.., '.'), ".")
        .map(|title: &str| vec![t(title.trim())])
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
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.is_part_of.unwrap().issue_number.is_none());

        // Without pages
        let reference = apa(&mut "Jones, A. (2021). New findings. Nature, 500(1).")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.page_start.is_none());
        assert!(reference.page_end.is_none());

        // Without DOI
        let reference = apa(&mut "Brown, K. (2019). Analysis. Medical Journal, 25(3) 10-20.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
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
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(reference.authors.unwrap().len(), 1);

        // Organization as author
        let reference = apa(
            &mut "World Health Organization (2020). Global report. Health Affairs, 10(5) 25-50.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));

        // Bare DOI
        let reference =
            apa(&mut "Clark, P. (2021). Findings. Tech Review, 3(1) 15-25. 10.1234/example")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.doi.is_some());

        // Prefixed DOI
        let reference = apa(
            &mut "Miller, L. (2019). Innovation. Future Studies, 7(2) 30-45. doi:10.5678/test",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.doi.is_some());

        Ok(())
    }

    #[test]
    fn test_book() -> Result<()> {
        // Canonical book example
        let reference = apa(
            &mut "Smith, J. A. (2020). Research Methods in Psychology. Academic Press. https://doi.org/10.1234/example",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.authors.is_some());
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Research Methods in Psychology".to_string())
        );
        assert!(reference.publisher.is_some());
        assert!(reference.doi.is_some());

        // Without DOI
        let reference =
            apa(&mut "Brown, K. (2019). Data Analysis Techniques. Science Publications.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.doi.is_none());

        // Multiple authors
        let reference =
            apa(&mut "Wilson, M., & Davis, R. (2021). Statistical Methods. Tech Press.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert_eq!(reference.authors.unwrap().len(), 2);

        // With extra whitespace
        let reference = apa(&mut "  Taylor, L.   ( 2018 ) .  Book Title  .  Publisher Name  .")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));

        Ok(())
    }

    #[test]
    fn test_chapter() -> Result<()> {
        // Test direct chapter parser first
        let reference = apa(
            &mut "Smith, J. A. (2020). Research methods. In Jones, B. C. (Ed.), Handbook of Psychology (15-30). Academic Press. https://doi.org/10.1234/example",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert!(reference.authors.is_some());
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Research methods".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .clone()
                .and_then(|book| book.title)
                .map(|title| to_text(&title)),
            Some("Handbook of Psychology".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .and_then(|book| book.publisher)
                .map(|publisher| publisher),
            Some(PersonOrOrganization::Organization(Organization {
                name: Some("Academic Press".to_string()),
                ..Default::default()
            }))
        );
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());
        assert!(reference.doi.is_some());

        // Multiple editors
        let reference = apa(
            &mut "Brown, K. (2019). Data analysis. In Wilson, M., & Davis, R. (Eds.), Statistical Methods (45-60). Publisher Name.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert!(reference.is_part_of.unwrap().editors.is_some());
        assert_eq!(reference.doi, None);

        // Without DOI
        let reference = apa(
            &mut "Taylor, L. (2021). New approaches. In Clark, P. (Ed.), Modern Techniques (100-120). Science Press.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert_eq!(reference.doi, None);

        // With extra whitespace
        let reference = apa(
            &mut "  Miller, A.   ( 2018 ) .  Chapter title  . In  Editor, E.  ( Ed. ) ,  Book Title  ( 5-10 ) .  Publisher  .",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));

        Ok(())
    }

    #[test]
    fn test_web() -> Result<()> {
        // Canonical web resource with author
        let reference = apa(
            &mut "Smith, J. A. (2023). Understanding web development. MDN Web Docs. https://developer.mozilla.org/guide",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert!(reference.authors.is_some());
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Understanding web development".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .and_then(|site| site.title)
                .map(|title| to_text(&title)),
            Some("MDN Web Docs".to_string())
        );
        assert!(reference.identifiers.is_some());

        // Without author (common for web resources)
        let reference =
            apa(&mut "(2024). Climate change report. EPA Website. https://epa.gov/climate")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert!(reference.authors.is_none());

        // With multiple authors
        let reference = apa(
            &mut "Wilson, M., & Davis, R. (2022). JavaScript best practices. Tech Blog. https://techblog.com/js-practices",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(reference.authors.unwrap().len(), 2);

        // With organization as author
        let reference = apa(
            &mut "World Health Organization (2023). Global health statistics. WHO Website. https://who.int/data/statistics",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));

        // With extra whitespace
        let reference = apa(
            &mut "  Brown, K.   ( 2021 ) .  Web accessibility guide  .  Accessibility Hub  .  https://a11y.com/guide  ",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));

        Ok(())
    }
}
