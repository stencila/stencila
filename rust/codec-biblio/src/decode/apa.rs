//! Parsers that parse Stencila [`Reference`] nodes from strings in APA reference list format
//!
//! This module provides parsers for extracting bibliographic information from APA
//! (American Psychological Association) style reference citations. The parsers handle
//! the standard components of APA references including authors, publication dates,
//! titles, journal information, volume/issue numbers, page ranges, and DOIs.

use winnow::{
    Parser, Result,
    ascii::{Caseless, digit1, multispace0, multispace1},
    combinator::{alt, delimited, opt, preceded, terminated},
    token::take_while,
};

use codec::schema::{
    CreativeWorkType, Date, Inline, IntegerOrString, Organization, PersonOrOrganization, Reference,
    shortcuts::t,
};

use crate::decode::{
    authors::{authors, persons},
    date::year,
    doi::doi_or_url,
    pages::pages,
    preprints::preprint_server,
    terminator::terminator,
    url::url,
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
#[allow(unused)]
pub fn apa(input: &mut &str) -> Result<Reference> {
    // Order is important for correct matching!
    // Most specific patterns first: chapter (has "In" keyword),
    // then article (has vol./issue), then web (quoted title),
    // then book (unquoted title)
    alt((chapter, article, web, book)).parse_next(input)
}

/// Parse an APA journal article reference
///
/// Parses APA-style journal article references with the following expected format:
///
/// ```text
/// Author, A. B., & Author, C. D. (Year). Title of article. Journal Name, Volume(Issue), pages. DOI
/// ```
pub fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse one or more authors (persons or organizations)
        authors,
        // Date: Parse year in parentheses format "(YYYY)"
        preceded(apa_separator, apa_year),
        // Title: Parse article title ending with a period
        preceded(apa_separator, apa_title),
        // Journal: Parse journal name
        preceded(
            apa_separator,
            alt((
                terminated(preprint_server, opt((multispace1, Caseless("preprint")))),
                take_while(1.., |c: char| c != ','),
            )),
        ),
        preceded(
            apa_separator,
            alt((
                (
                    (preprint_server, multispace0, ":", multispace0).map(|_| (None, None)),
                    take_while(1.., |c: char| !c.is_whitespace()).map(|id: &str| {
                        Some(Reference {
                            pagination: Some(id.trim_end_matches(['.', ',', ';']).to_string()),
                            ..Default::default()
                        })
                    }),
                ),
                (
                    // Volume and Issue: volume number with optional issue in parentheses
                    (
                        digit1.map(|vol| Some(vol)),
                        opt(delimited(
                            (multispace0, "(", multispace0),
                            digit1,
                            (multispace0, ")"),
                        )),
                    ),
                    // Pages: Optional page range
                    opt(preceded(
                        alt((apa_separator, (multispace0, ":", multispace0).take())),
                        apa_pages,
                    )),
                ),
            )),
        ),
        // DOI or URL
        opt(preceded(apa_separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, date, title, journal, ((volume, issue), pages), doi_or_url, _terminator)| {
                Reference {
                    work_type: Some(CreativeWorkType::Article),
                    authors: Some(authors),
                    date: Some(date),
                    title: Some(title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(journal)]),
                        volume_number: volume.map(IntegerOrString::from),
                        issue_number: issue.map(IntegerOrString::from),
                        ..Default::default()
                    })),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
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
pub fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse book authors
        authors,
        // Date: Parse year in parentheses format "(YYYY)"
        preceded(apa_separator, apa_year),
        // Title: Parse book title ending with a period
        preceded(apa_separator, apa_title),
        // Publisher: Parse publisher
        opt(preceded(apa_separator, take_while(1.., |c: char| c != '.'))),
        // DOI or URL
        opt(preceded(apa_separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        // Map the parsed components into a Reference struct
        .map(
            |(authors, date, title, publisher, doi_or_url, _terminator)| Reference {
                work_type: Some(CreativeWorkType::Book),
                authors: Some(authors),
                date: Some(date),
                title: Some(title),
                publisher: publisher.map(|publisher| {
                    PersonOrOrganization::Organization(Organization {
                        name: Some(publisher.trim().to_string()),
                        ..Default::default()
                    })
                }),
                doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                ..Default::default()
            },
        )
        .parse_next(input)
}

/// Parse an APA book chapter reference
///
/// Parses APA-style book chapter references with the following expected format:
///
/// ```text
/// Author, A. B. (Year). Chapter title. In Editor, E. D. (Ed.), Book title (pages). Publisher. DOI
/// ```
pub fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse chapter authors
        authors,
        // Date: Parse year in parentheses
        preceded(apa_separator, apa_year),
        // Chapter Title: Parse chapter title ending with period
        preceded(apa_separator, apa_title),
        // "In" keyword with space
        delimited(apa_separator, Caseless("In"), multispace1),
        // Editors: before (Ed.) or (Eds.)
        // Allows for variations such as (Ed) ( Eds) ( Ed. )
        opt(terminated(
            persons,
            (
                opt((
                    multispace0,
                    "(",
                    multispace0,
                    Caseless("Ed"),
                    opt("s"),
                    opt("."),
                    multispace0,
                    ")",
                    multispace0,
                )),
                (multispace0, ",", multispace0),
            ),
        )),
        // Book Title: Parse book title before opening parenthesis
        preceded(multispace0, take_while(1.., |c: char| c != '(')),
        // Pages: Parse page range in parentheses
        opt(delimited(
            (alt((apa_separator, multispace0)), "(", multispace0),
            pages,
            (multispace0, ")", opt((multispace0, "."))),
        )),
        // Publisher: Parse publisher
        opt(preceded(apa_separator, take_while(1.., |c: char| c != '.'))),
        // DOI or URL
        opt(preceded(apa_separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        // Map the parsed components into a Reference struct
        .map(
            |(
                authors,
                date,
                chapter_title,
                _in,
                editors,
                book_title,
                pages,
                publisher,
                doi_or_url,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    authors: Some(authors),
                    date: Some(date),
                    title: Some(chapter_title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(book_title.trim().to_string())]),
                        editors,
                        publisher: publisher.map(|publisher| {
                            PersonOrOrganization::Organization(Organization {
                                name: Some(publisher.trim().to_string()),
                                ..Default::default()
                            })
                        }),
                        ..Default::default()
                    })),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
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
pub fn web(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse web authors (may be missing for some web content)
        opt(terminated(authors, apa_separator)),
        // Date: Parse year in parentheses format "(YYYY)"
        apa_year,
        // Title: Parse web page title ending with a period
        preceded(apa_separator, apa_title),
        // Website: Parse website name
        opt(preceded(
            apa_separator,
            take_while(1.., |c: char| c != '.')
                .verify(|chars: &str| !chars.contains("https://") && !chars.contains("https://")),
        )),
        // URL: Web address
        preceded(
            (
                apa_separator,
                opt((
                    Caseless("Retrieved"),
                    opt((multispace1, Caseless("from"))),
                    opt((multispace0, ":")),
                    multispace0,
                )),
            ),
            url,
        ),
        // Optional terminator
        opt(terminator),
    )
        // Map the parsed components into a Reference struct
        .map(
            |(authors, date, title, website, url, _terminator)| Reference {
                work_type: Some(CreativeWorkType::WebPage),
                authors,
                date: Some(date),
                title: Some(title),
                is_part_of: website.map(|title| {
                    Box::new(Reference {
                        title: Some(vec![t(title)]),
                        ..Default::default()
                    })
                }),
                url: Some(url),
                ..Default::default()
            },
        )
        .parse_next(input)
}

/// Parse year in parentheses format "(YYYY)"
///
/// Allows optional whitespacewithin parentheses
fn apa_year(input: &mut &str) -> Result<Date> {
    delimited(("(", multispace0), year, (multispace0, ")")).parse_next(input)
}

/// Parse article title ending with a period
///
/// Captures everything up to the first period
fn apa_title(input: &mut &str) -> Result<Vec<Inline>> {
    take_while(1.., |c: char| c != '.')
        .map(|title: &str| vec![t(title.trim())])
        .parse_next(input)
}

/// Parse page numbers with APA formatting
fn apa_pages(input: &mut &str) -> Result<Reference> {
    pages.parse_next(input)
}

/// Parse a separator between parts of an APA reference
///
/// This is a lenient parser for anything that may be used as a separator
/// between parts of an APA reference. Making it lenient allows the `apa` parser
/// to be more robust to deviations in punctuation and whitespace.
fn apa_separator<'s>(input: &mut &'s str) -> Result<&'s str> {
    alt((
        (multispace0, alt((",", ".")), multispace0).take(),
        multispace1,
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use codec_text_trait::to_text;
    use common_dev::pretty_assertions::assert_eq;

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
            &mut "Author, A. B., & Author, C. D. (1999). Title of article. Title of Journal, 1(2) 34-56. https://doi.org/10.1234/abc",
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
        assert_eq!(reference.url, None);
        assert!(reference.doi.is_some());

        // Without issue number
        let reference = apa(&mut "Smith, J. (2020). Research methods. Science Journal, 15 45-60.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(
            reference
                .is_part_of
                .as_ref()
                .map(|part_of| part_of.issue_number.is_none())
                .unwrap_or(false)
        );

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
            &mut "Wilson, M.    (  2022  ) .   Title here  . Journal Name   , 12 (4  )  100-110.",
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
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(1));

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

        // Colon between volume/issue and pages
        let reference = apa(
            &mut "Anyaso-Samuel, S., Bandyopadhyay, D., and Datta, S. (2023). Pseudo-value regression of clustered multistate current status data with informative cluster sizes. Statistical Methods in Medical Research, 32(8):1494–1510.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(reference.page_end, Some(IntegerOrString::Integer(1510)));

        // arxiv as publisher
        let reference = apa(
            &mut "Anyaso-Samuel, S. and Datta, S. (2024). Nonparametric estimation of a future entry time distribution given the knowledge of a past state occupation in a progressive multistate model with current status data. arXiv preprint arXiv:2405.05781.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            reference.is_part_of,
            Some(Box::new(Reference {
                title: Some(vec![t("arXiv")]),
                ..Default::default()
            }))
        );
        assert_eq!(reference.pagination, Some("2405.05781".into()));

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
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));

        // With extra whitespace
        let reference = apa(&mut "Taylor, L.  ( 2018   ).   Book Title.    Publisher Name.")?;
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
            reference.is_part_of.and_then(|book| book.publisher),
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
        assert!(
            reference
                .is_part_of
                .as_ref()
                .map(|part_of| part_of.editors.is_some())
                .unwrap_or(false)
        );
        assert_eq!(reference.doi, None);

        // Without DOI
        let reference = apa(
            &mut "Taylor, L. (2021). New approaches. In Clark, P. (Ed.), Modern Techniques (100-120). Science Press.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert_eq!(reference.doi, None);

        // With extra whitespace
        let reference = apa(
            &mut "Miller, A.   ( 2018  ).  Chapter title. In Editor, E.  (Ed.) ,  Book Title   (5-10). Publisher.",
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
        assert!(reference.url.is_some());

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
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));

        // With organization as author
        let reference = apa(
            &mut "World Health Organization (2023). Global health statistics. WHO Website. https://who.int/data/statistics",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));

        // With extra whitespace, no website title
        let reference =
            apa(&mut "Brown, K. (2021 ).  Web accessibility guide.  https://a11y.com/guide")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(reference.url, Some("https://a11y.com/guide".into()));

        let reference = apa(
            &mut "Birla, N. (2019). Vehicle Dataset from CarDekho. Retrieved from: https://www.kaggle.com/datasets/nehalbirla/vehicle-dataset-from-cardekho",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(
            reference.url,
            Some("https://www.kaggle.com/datasets/nehalbirla/vehicle-dataset-from-cardekho".into())
        );

        Ok(())
    }
}
