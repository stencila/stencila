//! Parsers that parse Stencila [`Reference`] nodes from strings in ApJ reference list formats
//!
//! This module provides parsers for extracting bibliographic information from reference formats
//! commonly used in astrophysics and which have similar syntax
//!
//! - ApJ (The Astrophysical Journal)
//! - A&A (Astronomy & Astrophysics)
//! - MNRAS (Monthly Notices of the Royal Astronomical Society)

use winnow::{
    Parser, Result,
    ascii::{Caseless, digit1, multispace0},
    combinator::{alt, delimited, opt, preceded, separated},
    token::take_while,
};

use stencila_codec::stencila_schema::{
    Author, CreativeWorkType, Inline, IntegerOrString, Organization, Person, PersonOptions,
    Reference, ReferenceOptions, shortcuts::t,
};

use crate::decode::{
    parts::{
        authors::{authors, person_given_family},
        date::year_az,
        doi::doi_or_url,
        is_part_of::{in_book, in_journal, in_proceedings},
        pages::pages,
        publisher::place_publisher,
        separator::separator,
        terminator::terminator,
        url::url,
    },
    reference::generate_id,
};

/// Parse a Stencila [`Reference`] from an ApJ reference list item
///
/// This is the main entry point for parsing ApJ-style references. It attempts to
/// identify the type of reference and parse accordingly.
#[allow(unused)]
pub fn apj(input: &mut &str) -> Result<Reference> {
    // Order is important for correct matching!
    alt((chapter, conference, article, web, book)).parse_next(input)
}

/// Parse an ApJ journal article reference
///
/// ```text
/// Smith, P., Doe, J., & Black, A. 2019, Int. J. Examples, 12, 45
/// ```
pub fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Date
        preceded(separator, year_az),
        // Journal
        preceded(separator, take_while(1.., |c: char| c != ',')),
        // Volume
        preceded(separator, digit1),
        // Pages
        opt(preceded(separator, pages)),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, (date, suffix), journal, volume, pages, doi_or_url, _terminator)| {
                Reference {
                    work_type: Some(CreativeWorkType::Article),
                    id: Some(generate_id(&authors, &Some((date.clone(), suffix)))),
                    authors: Some(authors),
                    date: Some(date),
                    is_part_of: in_journal(
                        vec![t(journal.trim())],
                        Some(IntegerOrString::from(volume)),
                        None,
                    ),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an ApJ conference paper reference
///
/// ```text
/// Smith, P., Doe, J., & Black, A. 2020, in Proc. 10th Int. Conf. Examples, 123, (Berlin: Springer)
/// ```
pub fn conference(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Year
        preceded(separator, year_az),
        // "in" keyword
        preceded(separator, Caseless("in")),
        // Proceedings Title
        preceded(separator, take_while(1.., |c: char| c != ',')),
        // Pages
        opt(preceded(separator, pages)),
        // Publisher
        opt(preceded(
            separator,
            alt((
                delimited(("(", multispace0), place_publisher, (multispace0, ")")),
                place_publisher,
            )),
        )),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(
                authors,
                (date, suffix),
                _in,
                proceedings_title,
                pages,
                publisher,
                doi_or_url,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Article),
                    id: Some(generate_id(&authors, &Some((date.clone(), suffix)))),
                    authors: Some(authors),
                    date: Some(date),
                    is_part_of: in_proceedings(vec![t(proceedings_title.trim())], None, publisher),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an ApJ book chapter reference
///
/// ```text
/// Smith, P., Doe, J., & Black, A. 2017, Example chapter title, in Title of the Book, ed. A. Editor & B. Editor (Berlin: Springer), 45
/// ```
pub fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Year
        preceded(separator, year_az),
        // Chapter Title
        preceded(separator, apj_title),
        // "in" keyword
        preceded(separator, Caseless("in")),
        // Book Title
        preceded(separator, apj_book_title),
        // Editors
        opt(preceded(
            (
                separator,
                multispace0,
                Caseless("ed"),
                opt("s"),
                opt("."),
                multispace0,
            ),
            apj_editors,
        )),
        // Publisher
        opt(preceded(
            opt(separator),
            alt((
                delimited(("(", multispace0), place_publisher, (multispace0, ")")),
                place_publisher,
            )),
        )),
        // Pages
        opt(preceded(separator, pages)),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(
                authors,
                (date, suffix),
                chapter_title,
                _in,
                book_title,
                editors,
                publisher,
                pages,
                doi_or_url,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    id: Some(generate_id(&authors, &Some((date.clone(), suffix)))),
                    authors: Some(authors),
                    date: Some(date),
                    title: Some(chapter_title),
                    is_part_of: in_book(book_title, editors, publisher, None),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an ApJ book reference
///
/// ```text
/// Smith, P., & Doe, J. 2018, An Example Book (Berlin: Springer)
/// ```
pub fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Year
        preceded(separator, year_az),
        // Book Title
        preceded(separator, apj_book_title),
        // Publisher
        opt(preceded(
            opt(separator),
            alt((
                delimited(("(", multispace0), place_publisher, (multispace0, ")")),
                place_publisher,
            )),
        )),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, (date, suffix), title, publisher, doi_or_url, _terminator)| Reference {
                work_type: Some(CreativeWorkType::Book),
                id: Some(generate_id(&authors, &Some((date.clone(), suffix)))),
                authors: Some(authors),
                date: Some(date),
                title: Some(title),
                doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                options: Box::new(ReferenceOptions {
                    publisher,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .parse_next(input)
}

/// Parse an ApJ web resource reference
///
/// ```text
/// Smith, P., & Doe, J. 2021, Example web article title, ExampleWebsite.org, https://www.example.org/article
/// ```
pub fn web(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Year
        preceded(separator, year_az),
        // Title
        preceded(separator, apj_title),
        // Website Title
        opt(preceded(separator, apj_title)),
        // URL
        preceded(separator, url),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, (date, suffix), title, _site_title, url, _terminator)| Reference {
                work_type: Some(CreativeWorkType::WebPage),
                id: Some(generate_id(&authors, &Some((date.clone(), suffix)))),
                authors: Some(authors),
                date: Some(date),
                title: Some(title),
                url: Some(url),
                ..Default::default()
            },
        )
        .parse_next(input)
}

/// Parse title
fn apj_title(input: &mut &str) -> Result<Vec<Inline>> {
    take_while(1.., |c: char| c != ',')
        .map(|title: &str| vec![t(title.trim().trim_end_matches([',', '.']))])
        .parse_next(input)
}

/// Parse book title
fn apj_book_title(input: &mut &str) -> Result<Vec<Inline>> {
    take_while(1.., |c: char| c != ',' && c != '(')
        .map(|title: &str| vec![t(title.trim().trim_end_matches([',', '.']))])
        .parse_next(input)
}

/// Parse editors in ApJ formatting
fn apj_editors(input: &mut &str) -> Result<Vec<Person>> {
    separated(
        1..,
        person_given_family.map(|author| match author {
            Author::Person(person) => person,
            Author::Organization(Organization { name, .. }) => Person {
                options: Box::new(PersonOptions {
                    name,
                    ..Default::default()
                }),
                ..Default::default()
            },
            _ => Person::default(),
        }),
        (
            multispace0,
            alt((", and", ", &", "&", "and", ",")),
            multispace0,
        ),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use stencila_codec::stencila_schema::{
        OrganizationOptions, PersonOrOrganization, PostalAddressOrString,
    };
    use stencila_codec_text_trait::to_text;

    use super::*;

    // These tests call the top level `apj` parser to test for discrimination
    // between different work types.
    //
    // Avoid temptation to assert parsed details of works, instead relying on
    // the unit test for sub-parsers in other modules for that, where they exist.

    #[test]
    fn test_article() -> Result<()> {
        let reference =
            apj(&mut r#"Smith, P., Doe, J., & Black, A. 2019, Int. J. Examples, 12, 45"#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(3));
        assert!(reference.date.is_some());
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|journal| journal.title.as_ref())
                .map(to_text),
            Some("Int. J. Examples".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|part_of| part_of.options.volume_number.as_ref())
                .cloned(),
            Some(IntegerOrString::Integer(12))
        );
        assert_eq!(
            reference.options.page_start,
            Some(IntegerOrString::Integer(45))
        );

        // RAS / MNRAS style without comma after family name and comma after all authors
        let reference =
            article(&mut r#"Smith P. A., Doe J., & Black A., 2019, Int. J. Examples, 12, 45"#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        dbg!(&reference);
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(3));

        Ok(())
    }

    #[test]
    fn test_conference() -> Result<()> {
        let reference = apj(
            &mut r#"Smith, P., Doe, J., & Black, A. 2020, in Proc. 10th Int. Conf. Examples, 123, (Berlin: Springer)"#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(3));
        assert!(reference.date.is_some());
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|proceedings| proceedings.title.as_ref())
                .map(to_text),
            Some("Proc. 10th Int. Conf. Examples".to_string())
        );
        assert_eq!(
            reference.options.page_start,
            Some(IntegerOrString::Integer(123))
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|proceedings| proceedings.options.publisher.clone()),
            Some(PersonOrOrganization::Organization(Organization {
                name: Some("Springer".into()),
                options: Box::new(OrganizationOptions {
                    address: Some(PostalAddressOrString::String("Berlin".into())),
                    ..Default::default()
                }),
                ..Default::default()
            }))
        );

        Ok(())
    }

    #[test]
    fn test_chapter() -> Result<()> {
        let reference = apj(
            &mut r#"Smith, P., Doe, J., & Black, A. 2017, Example chapter title, in Title of the Book, ed. A. Editor & B. Editor (Berlin: Springer), 45"#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(3));
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Example chapter title".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|book| book.title.as_ref())
                .map(to_text),
            Some("Title of the Book".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .map(|book| book.options.editors.iter().flatten().count()),
            Some(2)
        );
        assert_eq!(
            reference.options.page_start,
            Some(IntegerOrString::Integer(45))
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|proceedings| proceedings.options.publisher.clone()),
            Some(PersonOrOrganization::Organization(Organization {
                name: Some("Springer".into()),
                options: Box::new(OrganizationOptions {
                    address: Some(PostalAddressOrString::String("Berlin".into())),
                    ..Default::default()
                }),
                ..Default::default()
            }))
        );

        Ok(())
    }

    #[test]
    fn test_book() -> Result<()> {
        let reference =
            apj(&mut r#"Smith, P., & Doe, J. 2018, An Example Book (Berlin: Springer)"#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("An Example Book".to_string())
        );
        assert_eq!(reference.is_part_of, None);
        assert_eq!(
            reference.options.publisher,
            Some(PersonOrOrganization::Organization(Organization {
                name: Some("Springer".into()),
                options: Box::new(OrganizationOptions {
                    address: Some(PostalAddressOrString::String("Berlin".into())),
                    ..Default::default()
                }),
                ..Default::default()
            }))
        );

        Ok(())
    }

    #[test]
    fn test_web() -> Result<()> {
        let reference = apj(
            &mut r#"Smith, P., & Doe, J. 2021, Example web article title, ExampleWebsite.org, https://www.example.org/article"#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));
        assert!(reference.date.is_some());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Example web article title".to_string())
        );
        assert_eq!(
            reference.url,
            Some("https://www.example.org/article".to_string())
        );

        Ok(())
    }
}
