//! Parsers that parse Stencila [`Reference`] nodes from strings in Chicago reference list format
//!
//! This module provides parsers for extracting bibliographic information from Chicago
//! (Chicago Manual of Style) bibliography citations. The parsers handle
//! the standard components of Chicago references including authors, titles, publisher
//! information, publication dates, volume/issue numbers, page ranges, and DOIs/URLs.

use std::str::FromStr;

use codec::schema::{Date, Inline};
use winnow::{
    Parser, Result,
    ascii::{Caseless, digit1, multispace0, multispace1},
    combinator::{alt, delimited, opt, preceded, terminated},
    token::{take_until, take_while},
};

use codec::schema::{
    CreativeWorkType, IntegerOrString, Organization, PersonOrOrganization, Reference, shortcuts::t,
};

use crate::decode::{
    authors::{authors, persons},
    doi::doi_or_url,
    pages::pages,
    url::url,
};

/// Parse a Stencila [`Reference`] from a Chicago reference list item
///
/// This is the main entry point for parsing Chicago-style references. It attempts to
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
pub fn chicago(input: &mut &str) -> Result<Reference> {
    // Order is important for correct matching!
    // Most specific patterns first: chapter (has "In" keyword),
    // then article (has vol./no.), then web (quoted title),
    // then book (unquoted title)
    alt((chapter, article, web, book)).parse_next(input)
}

// Placeholder parsers - will be implemented in subsequent phases
/// Parse a Chicago journal article reference
///
/// Parses Chicago-style journal article references with the following expected format:
///
/// ```text
/// Author Last, First. "Title of Article." Journal Name vol. Volume, no. Issue (Date): pages. DOI/URL.
/// ```
fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse one or more authors
        authors,
        // Title: Parse article title in quotes
        preceded(chicago_separator, chicago_quoted_title),
        // Journal: Parse journal name until " vol." or other separators
        preceded(
            chicago_separator,
            alt((
                take_until(1.., " vol."),
                take_until(1.., " ("),
                take_while(1.., |c: char| c != ','),
            )),
        ),
        // Volume: Required volume with "vol." prefix for articles
        preceded(chicago_separator, chicago_volume),
        // Issue: Optional issue with "no." prefix
        opt(preceded(chicago_separator, chicago_issue)),
        // Date: Publication date in parentheses (Year) or (Month Year)
        opt(preceded(
            chicago_separator,
            delimited(
                ("(", multispace0),
                take_while(1.., |c: char| c != ')'),
                (multispace0, ")"),
            ),
        )),
        // Pages: Optional page range with colon prefix
        opt(preceded((multispace0, ":", multispace0), chicago_pages)),
        // DOI or URL
        opt(preceded(chicago_separator, doi_or_url)),
    )
        .map(
            |(authors, title, journal, volume, issue, date, pages, doi_or_url)| Reference {
                work_type: Some(CreativeWorkType::Article),
                authors: Some(authors),
                title: Some(title),
                is_part_of: Some(Box::new(Reference {
                    title: Some(vec![t(journal.trim())]),
                    volume_number: Some(volume),
                    issue_number: issue,
                    ..Default::default()
                })),
                date: date.and_then(|d| Date::from_str(d).ok()),
                doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                ..pages.unwrap_or_default()
            },
        )
        .parse_next(input)
}

/// Parse a Chicago book reference
///
/// Parses Chicago-style book references with the following expected format:
///
/// ```text
/// Author Last, First. Book Title. Publisher, Year. DOI/URL.
/// ```
fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse one or more authors
        authors,
        // Title: Parse unquoted book title (ending with period)
        preceded(chicago_separator, chicago_title),
        // Publisher: Parse publisher ending with comma
        preceded(chicago_separator, take_while(1.., |c: char| c != ',')),
        // Year: Publication year
        preceded(
            chicago_separator,
            take_while(1.., |c: char| c != '.' && c != 'h'),
        ),
        // DOI or URL
        opt(preceded(chicago_separator, doi_or_url)),
    )
        .map(|(authors, title, publisher, year, doi_or_url)| Reference {
            work_type: Some(CreativeWorkType::Book),
            authors: Some(authors),
            title: Some(title),
            publisher: Some(PersonOrOrganization::Organization(Organization {
                name: Some(publisher.trim().to_string()),
                ..Default::default()
            })),
            date: Date::from_str(year.trim()).ok(),
            doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
            url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse a Chicago book chapter reference
///
/// Parses Chicago-style book chapter references with the following expected format:
///
/// ```text
/// Author Last, First. "Chapter Title." In Book Title, edited by Editor Name, pages. Publisher, Year.
/// ```
fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse chapter authors
        authors,
        // Chapter Title: Parse chapter title in quotes
        preceded(chicago_separator, chicago_quoted_title),
        // "In" keyword
        preceded(chicago_separator, "In"),
        // Book Title: Parse book title (unquoted) until comma
        preceded(multispace1, take_while(1.., |c: char| c != ',')),
        // Editors: preceded by "edited by"
        preceded((chicago_separator, "edited by", multispace1), persons),
        // Pages: Optional page range after comma
        opt(preceded(
            (chicago_separator, opt("pp."), multispace0),
            chicago_pages,
        )),
        // Publisher: Parse publisher ending with comma
        opt(preceded(
            chicago_separator,
            take_while(1.., |c: char| c != ','),
        )),
        // Year: Publication year
        opt(preceded(
            chicago_separator,
            take_while(1.., |c: char| c != '.'),
        )),
        // DOI or URL
        opt(preceded(chicago_separator, doi_or_url)),
    )
        .map(
            |(
                authors,
                chapter_title,
                _,
                book_title,
                editors,
                pages,
                publisher,
                year,
                doi_or_url,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    authors: Some(authors),
                    title: Some(chapter_title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(book_title.trim())]),
                        editors: Some(editors),
                        publisher: publisher.map(|pub_name| {
                            PersonOrOrganization::Organization(Organization {
                                name: Some(pub_name.trim().to_string()),
                                ..Default::default()
                            })
                        }),
                        ..Default::default()
                    })),
                    date: year.and_then(|y| Date::from_str(y.trim()).ok()),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse a Chicago web resource reference
///
/// Parses Chicago-style web resource references with the following expected format:
///
/// ```text
/// Author Last, First. "Title of Webpage." Website Name. Accessed Date. URL.
/// ```
fn web(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse web authors (optional)
        opt(terminated(authors, chicago_separator)),
        // Title: Parse web page title in quotes
        chicago_quoted_title,
        // Website: Parse website name
        preceded(chicago_separator, take_while(1.., |c: char| c != '.')),
        // Access date: Optional "Accessed Date" information
        opt(preceded(
            (chicago_separator, "Accessed", multispace1),
            take_while(1.., |c: char| c != '.'),
        )),
        // URL: Web address
        preceded(chicago_separator, url),
    )
        .map(|(authors, title, website, _access_date, url)| Reference {
            work_type: Some(CreativeWorkType::WebPage),
            authors,
            title: Some(title),
            is_part_of: Some(Box::new(Reference {
                title: Some(vec![t(website.trim())]),
                ..Default::default()
            })),
            url: Some(url),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse a separator between parts of a Chicago reference
///
/// This is a lenient parser for anything that may be used as a separator
/// between parts of a Chicago reference. Making it lenient allows the `chicago` parser
/// to be more robust to deviations in punctuation and whitespace.
fn chicago_separator<'s>(input: &mut &'s str) -> Result<&'s str> {
    alt((
        (multispace0, alt((",", ".")), multispace0).take(),
        multispace1,
    ))
    .parse_next(input)
}

/// Parse title in quotes format "Title"
fn chicago_quoted_title(input: &mut &str) -> Result<Vec<Inline>> {
    delimited(
        alt(("\"", "\u{201c}")),
        take_while(1.., |c: char| c != '"' && c != '\u{201d}'),
        alt(("\"", "\u{201d}")),
    )
    .map(|title: &str| vec![t(title.trim().trim_end_matches("."))])
    .parse_next(input)
}

/// Parse book title (no quotes, unquoted in plain text)
fn chicago_title(input: &mut &str) -> Result<Vec<Inline>> {
    take_until(1.., '.')
        .map(|title: &str| vec![t(title.trim())])
        .parse_next(input)
}

/// Parse volume number with "vol." prefix
fn chicago_volume(input: &mut &str) -> Result<IntegerOrString> {
    preceded(
        (Caseless("vol"), multispace0, opt("."), multispace0),
        digit1,
    )
    .map(IntegerOrString::from)
    .parse_next(input)
}

/// Parse issue number with "no." prefix  
fn chicago_issue(input: &mut &str) -> Result<IntegerOrString> {
    preceded((Caseless("no"), multispace0, opt("."), multispace0), digit1)
        .map(IntegerOrString::from)
        .parse_next(input)
}

/// Parse page numbers with Chicago formatting
fn chicago_pages(input: &mut &str) -> Result<Reference> {
    pages.parse_next(input)
}

#[cfg(test)]
mod tests {
    use codec_text_trait::to_text;

    use super::*;

    #[test]
    fn test_chicago_separator() -> Result<()> {
        assert_eq!(chicago_separator(&mut ", ")?.trim(), ",");
        assert_eq!(chicago_separator(&mut ". ")?.trim(), ".");
        assert_eq!(chicago_separator(&mut "  ")?.trim(), "");
        assert_eq!(chicago_separator(&mut " , ")?.trim(), ",");

        Ok(())
    }

    #[test]
    fn test_chicago_quoted_title() -> Result<()> {
        assert_eq!(
            chicago_quoted_title(&mut r#""The title""#)?,
            vec![t("The title")]
        );

        assert_eq!(
            chicago_quoted_title(&mut r#""The title.""#)?,
            vec![t("The title")]
        );

        // Test with smart quotes
        assert_eq!(
            chicago_quoted_title(&mut "\u{201c}Smart quotes\u{201d}")?,
            vec![t("Smart quotes")]
        );

        Ok(())
    }

    #[test]
    fn test_chicago_title() -> Result<()> {
        assert_eq!(chicago_title(&mut "Book Title.")?, vec![t("Book Title")]);

        assert_eq!(
            chicago_title(&mut "A Long Book Title with Many Words.")?,
            vec![t("A Long Book Title with Many Words")]
        );

        Ok(())
    }

    #[test]
    fn test_chicago_volume() -> Result<()> {
        assert_eq!(chicago_volume(&mut "vol. 1")?, IntegerOrString::Integer(1));
        assert_eq!(
            chicago_volume(&mut "vol . 123")?,
            IntegerOrString::Integer(123)
        );
        assert_eq!(
            chicago_volume(&mut "VOL 456")?,
            IntegerOrString::Integer(456)
        );

        Ok(())
    }

    #[test]
    fn test_chicago_issue() -> Result<()> {
        assert_eq!(chicago_issue(&mut "no. 1")?, IntegerOrString::Integer(1));
        assert_eq!(
            chicago_issue(&mut "no . 123")?,
            IntegerOrString::Integer(123)
        );
        assert_eq!(
            chicago_issue(&mut "NO   456")?,
            IntegerOrString::Integer(456)
        );

        Ok(())
    }

    #[test]
    fn test_article() -> Result<()> {
        // Without pages first (simpler case)
        let reference =
            article(&mut r#"Wilson, Mark. "New Discoveries." Nature vol. 500 (2021)."#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.page_start.is_none());

        // Test with pages
        let reference = article(
            &mut r#"Brown, Alice. "Research Methods." Science Journal vol. 10 (2020): 100-115."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());

        // Canonical Chicago article format with all components
        let reference = article(
            &mut r#"Smith, John. "Understanding Climate Change." Environmental Science vol. 15, no. 3 (2023): 45-67. https://doi.org/10.1234/example"#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert!(reference.authors.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Understanding Climate Change".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|journal| journal.title.as_ref())
                .map(to_text),
            Some("Environmental Science".to_string())
        );
        assert_eq!(
            reference.is_part_of.as_ref().and_then(|part_of| part_of.volume_number.as_ref()).cloned(),
            Some(IntegerOrString::Integer(15))
        );
        assert_eq!(
            reference.is_part_of.as_ref().and_then(|part_of| part_of.issue_number.as_ref()).cloned(),
            Some(IntegerOrString::Integer(3))
        );
        assert!(reference.date.is_some());
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());
        assert!(reference.doi.is_some());

        Ok(())
    }

    #[test]
    fn test_book() -> Result<()> {
        // Canonical Chicago book format
        let reference = book(
            &mut r#"Johnson, Maria. The Art of Programming. Tech Publications, 2022. https://doi.org/10.5678/book"#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.authors.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("The Art of Programming".to_string())
        );
        assert!(reference.publisher.is_some());
        assert!(reference.date.is_some());
        assert!(reference.doi.is_some());

        // Without DOI
        let reference = book(&mut r#"Davis, Robert. Statistical Analysis. Academic Press, 2019."#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.doi.is_none());

        // Multiple authors
        let reference =
            book(&mut r#"Smith, John, and Jane Doe. Research Methods. Science Publishers, 2021."#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));

        Ok(())
    }

    #[test]
    fn test_chapter() -> Result<()> {
        // Basic Chicago chapter format
        let reference = chapter(
            &mut r#"Taylor, Sarah. "Modern Techniques." In Handbook of Methods, edited by Peter Clark, 25-40. University Press, 2021. https://doi.org/10.9012/chapter"#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert!(reference.authors.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Modern Techniques".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|book| book.title.as_ref())
                .map(to_text),
            Some("Handbook of Methods".to_string())
        );
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());
        assert!(reference.doi.is_some());

        // Without DOI or pages
        let reference = chapter(
            &mut r#"Chen, Mei-Ling. "Kinship Studies." In Family Research, edited by Laura Mitchell. Chicago Press, 2023."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert!(reference.page_start.is_none());
        assert!(reference.doi.is_none());

        // Multiple authors and editors
        let reference = chapter(
            &mut r#"Patel, Riya, and David Ross. "Computing Systems." In Technology Handbook, edited by Irene Alvarez and Tomoko Sato, 89-117. Springer, 2022."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));
        assert!(reference.is_part_of.as_ref().and_then(|part_of| part_of.editors.as_ref()).map(|editors| !editors.is_empty()).unwrap_or(false));

        Ok(())
    }

    #[test]
    fn test_web() -> Result<()> {
        // Canonical Chicago web format with author
        let reference = web(
            &mut r#"Miller, Lisa. "Web Development Guide." Tech Resources. Accessed January 15, 2024. https://example.com/guide"#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert!(reference.authors.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Web Development Guide".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|site| site.title.as_ref())
                .map(to_text),
            Some("Tech Resources".to_string())
        );
        assert!(reference.url.is_some());

        // Without author
        let reference =
            web(&mut r#""Climate Report." Environmental Agency. https://epa.gov/climate"#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert!(reference.authors.is_none());

        // Without access date
        let reference =
            web(&mut r#"Brown, Kevin. "Accessibility Guide." A11Y Hub. https://a11y.com/guide"#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert!(reference.url.is_some());

        Ok(())
    }

    #[test]
    fn test_chicago_integration() -> Result<()> {
        // Test that the main chicago() parser correctly discriminates between types

        // Should parse as Chapter (has "In" keyword)
        let reference = chicago(
            &mut r#"Taylor, Sarah. "Modern Techniques." In Handbook of Methods, edited by Peter Clark, 25-40. University Press, 2021."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));

        // Should parse as Article (has volume/issue)
        let reference = chicago(
            &mut r#"Smith, John. "Understanding Climate Change." Environmental Science vol. 15, no. 3 (2023): 45-67."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));

        // Should parse as WebPage (quoted title, no vol./In)
        let reference = chicago(
            &mut r#"Miller, Lisa. "Web Development Guide." Tech Resources. https://example.com/guide"#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));

        // Should parse as Book (unquoted title)
        let reference =
            chicago(&mut r#"Johnson, Maria. The Art of Programming. Tech Publications, 2022."#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));

        Ok(())
    }
}
