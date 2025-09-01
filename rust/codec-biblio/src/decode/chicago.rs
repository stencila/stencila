//! Parsers that parse Stencila [`Reference`] nodes from strings in Chicago reference list format
//!
//! This module provides parsers for extracting bibliographic information from Chicago
//! (Chicago Manual of Style) bibliography citations. The parsers handle
//! the standard components of Chicago references including authors, titles, publisher
//! information, publication dates, volume/issue numbers, page ranges, and DOIs/URLs.

use codec::schema::Date;
use winnow::{
    Parser, Result,
    ascii::{Caseless, multispace0, multispace1},
    combinator::{alt, delimited, opt, preceded, terminated},
    token::{take_until, take_while},
};

use codec::schema::{
    CreativeWorkType, Organization, PersonOrOrganization, Reference, shortcuts::t,
};

use crate::decode::{
    parts::{
        authors::{authors, persons},
        date::year_az,
        doi::doi_or_url,
        pages::pages,
        separator::separator,
        terminator::terminator,
        title::{title_period_terminated, title_quoted},
        url::url,
        volume::{no_prefixed_issue, vol_prefixed_volume},
    },
    reference::generate_id,
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
#[allow(unused)]
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
pub fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse one or more authors
        authors,
        // Title
        preceded(separator, title_quoted),
        // Journal: Parse journal name until " vol." or other separators
        preceded(
            separator,
            alt((
                take_until(1.., " vol."),
                take_until(1.., " ("),
                take_while(1.., |c: char| c != ','),
            )),
        ),
        // Volume
        preceded(separator, vol_prefixed_volume),
        // Issue
        opt(preceded(separator, no_prefixed_issue)),
        // Date: Publication date in parentheses (Year) or (Month Year)
        opt(preceded(
            separator,
            delimited(
                ("(", multispace0),
                take_while(1.., |c: char| c != ')').map(|date: &str| Date::new(date.into())),
                (multispace0, ")"),
            ),
        )),
        // Pages: Optional page range with colon prefix
        opt(preceded((multispace0, ":", multispace0), pages)),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, title, journal, volume, issue, date, pages, doi_or_url, _terminator)| {
                Reference {
                    work_type: Some(CreativeWorkType::Article),
                    id: Some(generate_id(
                        &authors,
                        &date.clone().map(|date| (date, None)),
                    )),
                    authors: Some(authors),
                    title: Some(title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(journal.trim())]),
                        volume_number: Some(volume),
                        issue_number: issue,
                        ..Default::default()
                    })),
                    date,
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
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
pub fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Title
        preceded(separator, title_period_terminated),
        // Publisher
        opt(preceded(separator, take_while(1.., |c: char| c != ','))),
        // Year
        preceded(separator, year_az),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, title, publisher, (date, date_suffix), doi_or_url, _terminator)| Reference {
                work_type: Some(CreativeWorkType::Book),
                id: Some(generate_id(&authors, &Some((date.clone(), date_suffix)))),
                authors: Some(authors),
                title: Some(title),
                publisher: publisher.map(|publisher| {
                    PersonOrOrganization::Organization(Organization {
                        name: Some(publisher.trim().to_string()),
                        ..Default::default()
                    })
                }),
                date: Some(date),
                doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                ..Default::default()
            },
        )
        .parse_next(input)
}

/// Parse a Chicago book chapter reference
///
/// Parses Chicago-style book chapter references with the following expected format:
///
/// ```text
/// Author Last, First. "Chapter Title." In Book Title, edited by Editor Name, pages. Publisher, Year.
/// ```
pub fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse chapter authors
        authors,
        // Chapter Title
        preceded(separator, title_quoted),
        // "In" keyword
        preceded(separator, Caseless("In")),
        // Book Title: Parse book title (unquoted) until comma
        preceded(multispace1, take_while(1.., |c: char| c != ',')),
        // Editors: preceded by "edited by"
        opt(preceded(
            (separator, Caseless("edited by"), multispace1),
            persons,
        )),
        // Pages: Optional page range after comma
        opt(preceded(separator, pages)),
        // Publisher: Parse publisher ending with comma
        opt(preceded(separator, take_while(1.., |c: char| c != ','))),
        // Year: Publication year
        preceded(separator, year_az),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(
                authors,
                chapter_title,
                _in,
                book_title,
                editors,
                pages,
                publisher,
                (date, date_suffix),
                doi_or_url,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    id: Some(generate_id(&authors, &Some((date.clone(), date_suffix)))),
                    authors: Some(authors),
                    title: Some(chapter_title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(book_title.trim())]),
                        editors,
                        publisher: publisher.map(|pub_name| {
                            PersonOrOrganization::Organization(Organization {
                                name: Some(pub_name.trim().to_string()),
                                ..Default::default()
                            })
                        }),
                        ..Default::default()
                    })),
                    date: Some(date),
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
pub fn web(input: &mut &str) -> Result<Reference> {
    (
        // Authors: Parse web authors (optional)
        opt(terminated(authors, separator)),
        // Title
        title_quoted,
        // Website: Parse website name
        preceded(separator, take_while(1.., |c: char| c != '.')),
        // Access date: Optional "Accessed Date" information
        opt(preceded(
            (separator, Caseless("Accessed"), multispace1),
            take_while(1.., |c: char| c != '.'),
        )),
        // URL: Web address
        preceded(separator, url),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, title, website, _access_date, url, _terminator)| Reference {
                work_type: Some(CreativeWorkType::WebPage),
                id: authors.as_ref().map(|authors| generate_id(authors, &None)),
                authors,
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

#[cfg(test)]
mod tests {
    use codec::schema::IntegerOrString;
    use codec_text_trait::to_text;
    use pretty_assertions::assert_eq;

    use super::*;

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
            reference
                .is_part_of
                .as_ref()
                .and_then(|part_of| part_of.volume_number.as_ref())
                .cloned(),
            Some(IntegerOrString::Integer(15))
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|part_of| part_of.issue_number.as_ref())
                .cloned(),
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

        // Multiple authors and editors
        let reference = chapter(
            &mut r#"Patel, Riya, and David Ross. "Computing Systems." In Technology Handbook, edited by Irene Alvarez and Tomoko Sato, 89-117. Springer, 2022."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));
        assert!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|part_of| part_of.editors.as_ref())
                .map(|editors| !editors.is_empty())
                .unwrap_or(false)
        );

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
