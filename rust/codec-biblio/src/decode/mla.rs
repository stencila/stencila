//! Parsers that parse Stencila [`Reference`] nodes from strings in MLA reference list format
//!
//! This module provides parsers for extracting bibliographic information from MLA
//! (Modern Language Association) style reference citations. The parsers handle
//! the standard components of MLA references including authors, titles, container
//! information, publication dates, and URLs.

use std::str::FromStr;

use codec::schema::{Date, Inline};
use winnow::{
    Parser, Result,
    ascii::{digit1, multispace0, multispace1},
    combinator::{alt, delimited, opt, preceded},
    token::{take_until, take_while},
};

use codec::schema::{
    CreativeWorkType, IntegerOrString, Organization, PersonOrOrganization, Reference, shortcuts::t,
};

use crate::decode::{
    authors::{authors, persons},
    date::year,
    doi::doi_or_url,
    pages::pages,
    url::url,
};

/// Parse a Stencila [`Reference`] from an MLA reference list item
///
/// This is the main entry point for parsing MLA-style references. It attempts to
/// identify the type of reference and parse accordingly.
///
/// Currently supported reference types:
///
/// - Journal articles
/// - Books
/// - Book chapters
/// - Web resources
///
/// Future work may include newspapers, magazines, conference papers, etc.
pub fn mla(input: &mut &str) -> Result<Reference> {
    alt((article, chapter, web, book)).parse_next(input)
}

/// Parse an MLA journal article reference
///
/// Parses MLA-style journal article references with the following expected format:
///
/// ```text
/// Author, First. "Title of Article." Journal Name, vol. Volume, no. Issue, Year, pp. Pages. DOI/URL
/// ```
fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse one or more authors
        preceded(multispace0, authors),
        // Title: Parse article title in quotes
        mla_quoted_title,
        // Journal: Parse journal name ending with comma
        delimited(multispace0, take_until(1.., ','), ","),
        // Volume: Optional volume with "vol." prefix
        opt(delimited(
            multispace0,
            preceded(alt(("vol. ", "vol.", "Vol. ", "Vol.")), digit1),
            alt((",", multispace0)),
        )),
        // Issue: Optional issue with "no." prefix
        opt(delimited(
            multispace0,
            preceded(alt(("no. ", "no.", "No. ", "No.")), digit1),
            alt((",", multispace0)),
        )),
        // Year: Publication year
        delimited(multispace0, year, alt((",", multispace0))),
        // Pages: Optional page range with "pp." or "p." prefix
        opt(delimited(
            multispace0,
            preceded(alt(("pp. ", "pp.", "p. ", "p.")), pages),
            alt((".", ",", multispace0)),
        )),
        // DOI or URL
        opt(delimited(multispace0, doi_or_url, alt((".", multispace0)))),
    )
        .map(
            |(authors, title, journal, volume, issue, year, pages, doi_or_url)| Reference {
                work_type: Some(CreativeWorkType::Article),
                authors: Some(authors),
                title: Some(title),
                is_part_of: Some(Box::new(Reference {
                    title: Some(vec![t(journal.trim())]),
                    volume_number: volume.map(IntegerOrString::from),
                    issue_number: issue.map(IntegerOrString::from),
                    ..Default::default()
                })),
                date: Some(year),
                doi: doi_or_url
                    .as_ref()
                    .and_then(|doi_or_url| doi_or_url.doi.clone()),
                url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                ..pages.unwrap_or_default()
            },
        )
        .parse_next(input)
}

/// Parse an MLA book reference
///
/// Parses MLA-style book references with the following expected format:
///
/// ```text
/// Author, First. Title of Book. Publisher, Year. DOI/URL
/// ```
fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse book authors
        preceded(multispace0, authors),
        // Title: Parse book title (italicized/underlined in print)
        mla_unquoted_title,
        // Publisher: Parse publisher ending with comma
        delimited(multispace0, take_until(1.., ','), ","),
        // Year: Publication year
        delimited(multispace0, year, alt((".", multispace0))),
        // DOI or URL
        opt(delimited(multispace0, doi_or_url, alt((".", multispace0)))),
    )
        .map(|(authors, title, publisher, year, doi_or_url)| Reference {
            work_type: Some(CreativeWorkType::Book),
            authors: Some(authors),
            title: Some(title),
            publisher: Some(PersonOrOrganization::Organization(Organization {
                name: Some(publisher.trim().to_string()),
                ..Default::default()
            })),
            date: Some(year),
            doi: doi_or_url
                .as_ref()
                .and_then(|doi_or_url| doi_or_url.doi.clone()),
            url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse an MLA book chapter reference
///
/// Parses MLA-style book chapter references with the following expected format:
///
/// ```text
/// Author, First. "Chapter Title." Book Title, edited by Editor Name, Publisher, Year, pp. Pages. DOI/URL
/// ```
fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse chapter authors
        preceded(multispace0, authors),
        // Chapter Title: Parse chapter title in quotes
        mla_quoted_title,
        // Book Title: Parse book title before comma
        delimited(multispace0, take_until(1.., ','), ","),
        // Editors: with "edited by" prefix
        opt(delimited(
            multispace0,
            preceded(alt(("edited by ", "edited by")), persons),
            alt((",", multispace1)),
        )),
        // Publisher: Parse publisher ending with comma
        delimited(multispace0, take_until(1.., ','), ","),
        // Year: Publication year
        delimited(multispace0, year, alt((",", multispace1))),
        // Pages: Optional page range with "pp." prefix
        opt(delimited(
            multispace0,
            preceded(alt(("pp. ", "pp.", "p. ", "p.")), pages),
            alt((".", ",", multispace1)),
        )),
        // DOI or URL
        opt(delimited(multispace0, doi_or_url, alt((".", multispace0)))),
    )
        .map(
            |(authors, chapter_title, book_title, editors, publisher, year, pages, doi_or_url)| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    authors: Some(authors),
                    title: Some(chapter_title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(book_title.trim())]),
                        editors,
                        publisher: Some(PersonOrOrganization::Organization(Organization {
                            name: Some(publisher.trim().to_string()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    })),
                    date: Some(year),
                    doi: doi_or_url
                        .as_ref()
                        .and_then(|doi_or_url| doi_or_url.doi.clone()),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an MLA web resource reference
///
/// Parses MLA-style web resource references with the following expected format:
///
/// ```text
/// Author, First. "Title of Webpage." Website Name, Date, URL. Accessed Date.
/// ```
fn web(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse web authors (may be missing)
        opt(preceded(multispace0, authors)),
        // Title: Parse web page title in quotes
        mla_quoted_title,
        // Website: Parse website name ending with comma
        delimited(multispace0, take_until(1.., ','), ","),
        // Date: Publication date or year
        delimited(multispace0, alt((digit1, take_until(1.., ','))), ","),
        // URL: Web address
        preceded(multispace0, url),
        // Access date: Optional "Accessed Date" information
        opt(preceded(
            multispace0,
            preceded("Accessed ", take_while(1.., |c: char| c != '.')),
        )),
    )
        .map(
            |(authors, title, website, date, url, _access_date)| Reference {
                work_type: Some(CreativeWorkType::WebPage),
                authors,
                date: Date::from_str(date).ok(),
                title: Some(title),
                is_part_of: Some(Box::new(Reference {
                    title: Some(vec![t(website.trim())]),
                    ..Default::default()
                })),
                url: Some(url),
                ..Default::default()
            },
        )
        .parse_next(input)
}

/// Parse title in quotes format "Title"
fn mla_quoted_title(input: &mut &str) -> Result<Vec<Inline>> {
    delimited(
        (multispace0, "\""),
        take_until(1.., '"'),
        ("\"", opt((multispace0, "."))),
    )
    .map(|title: &str| vec![t(title.trim())])
    .parse_next(input)
}

/// Parse book title (no quotes, often italicized in print)
fn mla_unquoted_title(input: &mut &str) -> Result<Vec<Inline>> {
    delimited(multispace0, take_until(1.., '.'), ".")
        .map(|title: &str| vec![t(title.trim())])
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use codec_text_trait::to_text;

    use super::*;

    #[test]
    fn test_article() -> Result<()> {
        // Canonical example with all components
        let reference = mla(
            &mut "Smith, John A., and Jane B. Doe. \"Understanding Climate Change.\" Environmental Science, vol. 15, no. 3, 2023, pp. 45-67. https://doi.org/10.1234/example",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.authors.is_some());
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Understanding Climate Change".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .and_then(|journal| journal.title)
                .map(|title| to_text(&title)),
            Some("Environmental Science".to_string())
        );
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());
        assert!(reference.doi.is_some());

        // Without issue number
        let reference = mla(
            &mut "Brown, Alice. \"Research Methods.\" Science Journal, vol. 10, 2020, pp. 100-115.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.is_part_of.unwrap().issue_number.is_none());

        // Without pages
        let reference = mla(&mut "Wilson, Mark. \"New Discoveries.\" Nature, vol. 500, 2021.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.page_start.is_none());

        Ok(())
    }

    #[test]
    fn test_book() -> Result<()> {
        // Canonical book example
        let reference = mla(
            &mut "Johnson, Maria. The Art of Programming. Tech Publications, 2022. https://doi.org/10.5678/book",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.authors.is_some());
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("The Art of Programming".to_string())
        );
        assert!(reference.publisher.is_some());
        assert!(reference.doi.is_some());

        // Without DOI
        let reference = mla(&mut "Davis, Robert. Statistical Analysis. Academic Press, 2019.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.doi.is_none());

        Ok(())
    }

    #[test]
    fn test_chapter() -> Result<()> {
        let reference = mla(
            &mut "Taylor, Sarah. \"Modern Techniques.\" Handbook of Methods, edited by Peter Clark, University Press, 2021, pp. 25-40. https://doi.org/10.9012/chapter",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert!(reference.authors.is_some());
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Modern Techniques".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .clone()
                .and_then(|book| book.title)
                .map(|title| to_text(&title)),
            Some("Handbook of Methods".to_string())
        );
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());
        assert!(reference.doi.is_some());

        Ok(())
    }

    #[test]
    fn test_web() -> Result<()> {
        let reference = mla(
            &mut "Miller, Lisa. \"Web Development Guide.\" Tech Resources, 2023, https://example.com/guide. Accessed 15 Jan. 2024.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert!(reference.authors.is_some());
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Web Development Guide".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .and_then(|site| site.title)
                .map(|title| to_text(&title)),
            Some("Tech Resources".to_string())
        );
        assert!(reference.identifiers.is_some());

        // Without author
        let reference =
            mla(&mut "\"Climate Report.\" Environmental Agency, 2024, https://epa.gov/climate.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert!(reference.authors.is_none());

        Ok(())
    }
}
