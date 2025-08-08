//! Parsers that parse Stencila [`Reference`] nodes from strings in Vancouver reference list format
//!
//! This module provides parsers for extracting bibliographic information from Vancouver
//! (numbered) style reference citations. The parsers handle the standard components of
//! Vancouver references including authors, titles, journal information, publication dates,
//! volume/issue numbers, page ranges, and URLs/DOIs.

use codec::schema::Inline;
use winnow::{
    Parser, Result,
    ascii::{Caseless, digit1, multispace0, multispace1},
    combinator::{alt, delimited, opt, preceded, terminated},
    token::take_while,
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

// Note: Vancouver-style author parsing is handled by the generic authors parser
// with careful formatting using double periods (..) to terminate author names.
// Future improvement: implement Vancouver-specific author parsing to handle
// the native "Smith J, Jones A" format without format workarounds.

/// Parse a Stencila [`Reference`] from a Vancouver reference list item
///
/// This is the main entry point for parsing Vancouver-style references. It attempts to
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
pub fn vancouver(input: &mut &str) -> Result<Reference> {
    // Order is important for correct matching!
    // Most specific patterns first: chapter (has "In:" keyword),
    // then article (has journal;volume pattern), then web (has [Internet] marker),
    // then book (place:publisher pattern)
    alt((chapter, article, web, book)).parse_next(input)
}

/// Parse a Vancouver journal article reference
///
/// Parses Vancouver-style journal article references with the following expected format:
///
/// ```text
/// Author Initial. Title of article. Journal Name. Year;Volume(Issue):Pages.
/// ```
fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse one or more authors terminated before title
        authors,
        // Title: Parse article title ending with period
        preceded(vancouver_separator, vancouver_title),
        // Journal: Parse journal name ending with period
        preceded(vancouver_separator, vancouver_title),
        // Year: Publication year
        preceded(vancouver_separator, year),
        // Semicolon separator before volume
        preceded(vancouver_separator, vancouver_volume),
        // Pages: Optional page range after colon
        opt(preceded(vancouver_separator, vancouver_pages)),
        // DOI or URL (optional)
        opt(preceded(vancouver_separator, doi_or_url)),
    )
        .map(
            |(authors, title, journal, date, (volume, issue), pages, doi_or_url)| Reference {
                work_type: Some(CreativeWorkType::Article),
                authors: Some(authors),
                title: Some(title),
                is_part_of: Some(Box::new(Reference {
                    title: Some(journal),
                    volume_number: Some(volume),
                    issue_number: issue,
                    ..Default::default()
                })),
                date: Some(date),
                doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                ..pages.unwrap_or_default()
            },
        )
        .parse_next(input)
}

/// Parse a Vancouver book reference
///
/// Parses Vancouver-style book references with the following expected format:
///
/// ```text
/// Author Initial. Book Title. Place: Publisher; Year.
/// ```
fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse one or more authors terminated before title
        authors,
        // Title: Parse book title ending with period
        preceded(vancouver_separator, vancouver_title),
        // Place: Publisher: Parse place and publisher with colon separator
        preceded(vancouver_separator, vancouver_publisher),
        // Year: Publication year after semicolon
        preceded(vancouver_separator, year),
        // DOI or URL (optional)
        opt(preceded(vancouver_separator, doi_or_url)),
    )
        .map(
            |(authors, title, (_place, publisher), date, doi_or_url)| Reference {
                work_type: Some(CreativeWorkType::Book),
                authors: Some(authors),
                date: Some(date),
                title: Some(title),
                publisher: Some(publisher),
                doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                ..Default::default()
            },
        )
        .parse_next(input)
}

/// Parse a Vancouver book chapter reference
///
/// Parses Vancouver-style book chapter references with the following expected format:
///
/// ```text
/// Author Initial. Chapter Title. In: Editor Initial, editor. Book Title. Place: Publisher; Year. p. Pages.
/// ```
fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse chapter authors terminated before title
        authors,
        // Chapter Title: Parse chapter title ending with period
        preceded(vancouver_separator, vancouver_title),
        // "In:" keyword
        preceded(vancouver_separator, "In:"),
        // Editors: Parse editors after "In:"
        preceded(vancouver_separator, persons),
        // Book Title: Parse book title after editors
        preceded(vancouver_separator, vancouver_title),
        // Place: Publisher: Parse place and publisher
        preceded(vancouver_separator, vancouver_publisher),
        // Year: Publication year after semicolon
        preceded(vancouver_separator, year),
        // Pages: Optional pages with "p." prefix
        opt(preceded(
            (vancouver_separator, alt(("p.", "pp.")), multispace0),
            vancouver_pages,
        )),
        // DOI or URL (optional)
        opt(preceded(vancouver_separator, doi_or_url)),
    )
        .map(
            |(
                authors,
                chapter_title,
                _,
                editors,
                book_title,
                (_place, publisher),
                date,
                pages,
                doi_or_url,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    authors: Some(authors),
                    date: Some(date),
                    title: Some(chapter_title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(book_title),
                        editors: Some(editors),
                        publisher: Some(publisher),
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

/// Parse a Vancouver web resource reference
///
/// Parses Vancouver-style web resource references with the following expected format:
///
/// ```text
/// Author Initial. Title [Internet]. Available from: URL [cited Date].
/// ```
fn web(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse web authors (optional)
        opt(terminated(authors, vancouver_separator)),
        // Title: Parse web page title
        vancouver_title,
        // [Internet] marker
        preceded(
            vancouver_separator,
            ("[", multispace0, Caseless("internet"), multispace0, "]"),
        ),
        // "Available from:" prefix
        preceded((vancouver_separator, "Available from:", multispace0), url),
        // Citation date: Optional "[cited Date]" information
        opt((
            "[",
            multispace0,
            Caseless("cited"),
            multispace0,
            take_while(1.., |c: char| c != ']'),
            "]",
        )),
    )
        .map(|(authors, title, _, url, _cite_date)| Reference {
            work_type: Some(CreativeWorkType::WebPage),
            authors,
            title: Some(title),
            url: Some(url),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse a separator between parts of a Vancouver reference
///
/// This is a lenient parser for anything that may be used as a separator
/// between parts of a Vancouver reference. Making it lenient allows the `vancouver` parser
/// to be more robust to deviations in punctuation and whitespace.
fn vancouver_separator<'s>(input: &mut &'s str) -> Result<&'s str> {
    alt((
        (multispace0, alt((".", ";", ":")), multispace0).take(),
        multispace1,
    ))
    .parse_next(input)
}

/// Parse title format for Vancouver references
///
/// Vancouver titles are typically plain text ending with a period
fn vancouver_title(input: &mut &str) -> Result<Vec<Inline>> {
    take_while(1.., |c: char| c != '.')
        .map(|title: &str| vec![t(title.trim())])
        .parse_next(input)
}

/// Parse volume and issue in Vancouver format (Volume(Issue))
///
/// Vancouver format: Year;Volume(Issue) - this parses the Volume(Issue) part
fn vancouver_volume(input: &mut &str) -> Result<(IntegerOrString, Option<IntegerOrString>)> {
    (
        digit1, // Volume number (required)
        opt(delimited(
            (multispace0, "(", multispace0),
            digit1, // Issue number (optional)
            (multispace0, ")"),
        )),
    )
        .map(|(volume, issue)| {
            (
                IntegerOrString::from(volume),
                issue.map(IntegerOrString::from),
            )
        })
        .parse_next(input)
}

/// Parse page numbers with Vancouver formatting
///
/// Vancouver pages come directly after a colon (:) in articles
fn vancouver_pages(input: &mut &str) -> Result<Reference> {
    pages.parse_next(input)
}

/// Parse place and publisher in Vancouver format
///
/// Parses "Place: Publisher" or just "Publisher" format.
/// Returns a tuple of (optional place, publisher organization).
fn vancouver_publisher(input: &mut &str) -> Result<(Option<String>, PersonOrOrganization)> {
    take_while(1.., |c: char| c != ';')
        .map(|vancouver_publisher: &str| {
            if let Some(colon_pos) = vancouver_publisher.find(':') {
                let place_part = &vancouver_publisher[..colon_pos];
                let publisher_part = &vancouver_publisher[colon_pos + 1..];
                (
                    Some(place_part.trim().to_string()),
                    PersonOrOrganization::Organization(Organization {
                        name: Some(publisher_part.trim().to_string()),
                        ..Default::default()
                    }),
                )
            } else {
                (
                    None,
                    PersonOrOrganization::Organization(Organization {
                        name: Some(vancouver_publisher.trim().to_string()),
                        ..Default::default()
                    }),
                )
            }
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use codec_text_trait::to_text;

    use super::*;

    #[test]
    fn test_vancouver_separator() -> Result<()> {
        assert_eq!(vancouver_separator(&mut ". ")?.trim(), ".");
        assert_eq!(vancouver_separator(&mut "; ")?.trim(), ";");
        assert_eq!(vancouver_separator(&mut ": ")?.trim(), ":");
        assert_eq!(vancouver_separator(&mut "  ")?.trim(), "");
        assert_eq!(vancouver_separator(&mut " . ")?.trim(), ".");
        assert_eq!(vancouver_separator(&mut " ; ")?.trim(), ";");

        Ok(())
    }

    #[test]
    fn test_vancouver_title() -> Result<()> {
        assert_eq!(
            vancouver_title(&mut "Understanding climate change.")?,
            vec![t("Understanding climate change")]
        );

        assert_eq!(
            vancouver_title(&mut "  The effects of global warming  .")?,
            vec![t("The effects of global warming")]
        );

        assert_eq!(
            vancouver_title(&mut "Research methods in biomedical sciences.")?,
            vec![t("Research methods in biomedical sciences")]
        );

        Ok(())
    }

    #[test]
    fn test_vancouver_volume() -> Result<()> {
        // Volume only
        assert_eq!(
            vancouver_volume(&mut "15")?,
            (IntegerOrString::Integer(15), None)
        );

        // Volume with issue
        assert_eq!(
            vancouver_volume(&mut "15(3)")?,
            (
                IntegerOrString::Integer(15),
                Some(IntegerOrString::Integer(3))
            )
        );

        // Volume with issue and whitespace
        assert_eq!(
            vancouver_volume(&mut "15( 3 )")?,
            (
                IntegerOrString::Integer(15),
                Some(IntegerOrString::Integer(3))
            )
        );

        // Large volume and issue numbers
        assert_eq!(
            vancouver_volume(&mut "123(456)")?,
            (
                IntegerOrString::Integer(123),
                Some(IntegerOrString::Integer(456))
            )
        );

        Ok(())
    }

    #[test]
    fn test_vancouver_publisher() -> Result<()> {
        // Place and publisher with colon
        let (place, publisher) = vancouver_publisher(&mut "New York: Tech Press")?;
        assert_eq!(place, Some("New York".to_string()));
        if let PersonOrOrganization::Organization(org) = publisher {
            assert_eq!(org.name, Some("Tech Press".to_string()));
        } else {
            unreachable!("expected organization")
        }

        // Just publisher (no place)
        let (place, publisher) = vancouver_publisher(&mut "Academic Press")?;
        assert_eq!(place, None);
        if let PersonOrOrganization::Organization(org) = publisher {
            assert_eq!(org.name, Some("Academic Press".to_string()));
        } else {
            unreachable!("expected organization")
        }

        // With extra whitespace
        let (place, publisher) = vancouver_publisher(&mut "  Boston  :  University Press  ")?;
        assert_eq!(place, Some("Boston".to_string()));
        if let PersonOrOrganization::Organization(org) = publisher {
            assert_eq!(org.name, Some("University Press".to_string()));
        } else {
            unreachable!("expected organization")
        }

        Ok(())
    }

    #[test]
    fn test_vancouver_pages() -> Result<()> {
        // Single page
        let result = vancouver_pages(&mut "123")?;
        assert!(result.page_start.is_some());

        // Page range
        let result = vancouver_pages(&mut "123-456")?;
        assert!(result.page_start.is_some());
        assert!(result.page_end.is_some());

        // Page range with spaces
        let result = vancouver_pages(&mut "123 - 456")?;
        assert!(result.page_start.is_some());
        assert!(result.page_end.is_some());

        Ok(())
    }

    #[test]
    fn test_article() -> Result<()> {
        // Basic Vancouver article format with single author
        // Note: generic authors parser requires careful title formatting
        let reference =
            vancouver(&mut "Smith, J. A study on cancer prevention. BMJ. 2002;324(7337):577-81.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.authors.is_some());
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(1));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("A study on cancer prevention".to_string())
        );
        assert!(reference.is_part_of.is_some());
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|journal| journal.title.as_ref())
                .map(to_text),
            Some("BMJ".to_string())
        );
        assert!(reference.date.is_some());
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());

        // Multiple authors
        let reference = vancouver(
            &mut "Smith, J., Jones, A., & Brown, K. Multiple author study. Nature. 2023;500(1):15-30.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(3));

        // Without pages
        let reference = vancouver(&mut "Brown, K. Research methods. Science. 2022;10(3).")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.page_start.is_none());

        // Without issue number
        let reference =
            vancouver(&mut "Wilson, M. Data analysis. Journal of Statistics. 2021;15:45-67.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.is_part_of.as_ref().map(|part_of| part_of.issue_number.is_none()).unwrap_or(false));
        assert_eq!(
            reference.is_part_of.as_ref().and_then(|part_of| part_of.volume_number.as_ref()).cloned(),
            Some(IntegerOrString::Integer(15))
        );

        Ok(())
    }

    #[test]
    fn test_book() -> Result<()> {
        // Basic Vancouver book format with place and publisher
        let reference = vancouver(
            &mut "Smith, J., & Jones, A. Programming Guide. New York: Tech Press; 2023.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.authors.is_some());
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Programming Guide".to_string())
        );
        assert!(reference.publisher.is_some());
        assert!(reference.date.is_some());

        // Single author book
        let reference =
            vancouver(&mut "Brown, K.. Data Analysis Methods. Boston: Academic Press; 2022.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(1));

        // Book without place (just publisher)
        let reference =
            vancouver(&mut "Wilson, M. Statistical Computing. Science Publications; 2021.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.publisher.is_some());

        // Book with multiple family names
        let reference =
            vancouver(&mut "Van Der Berg, P. Advanced Topics. London: University Press; 2020.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.authors.is_some());

        Ok(())
    }
}
