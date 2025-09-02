//! Parsers that parse Stencila [`Reference`] nodes from strings in LNCS reference list format
//!
//! This module provides parsers for extracting bibliographic information from LNCS
//! (Lecture Notes in Computer Science) style reference citations.

use std::slice;

use winnow::{
    Parser, Result,
    ascii::{Caseless, digit1, multispace0, multispace1},
    combinator::{alt, delimited, opt, preceded, separated, terminated},
    token::take_while,
};

use stencila_codec::stencila_schema::{
    Author, CreativeWorkType, IntegerOrString, Organization, Person, PersonOptions, Reference,
    StringOrNumber, shortcuts::t,
};

use crate::decode::{
    parts::{
        authors::{authors, organization, person_given_family},
        date::year_az,
        doi::doi_or_url,
        pages::pages,
        publisher::publisher_place,
        separator::separator,
        terminator::terminator,
        title::title_period_terminated,
        url::url,
    },
    reference::generate_id,
};

/// Parse a Stencila [`Reference`] from an LNCS reference list item
///
/// This is the main entry point for parsing LNCS-style references. It attempts to
/// identify the type of reference and parse accordingly.
#[allow(unused)]
pub fn lncs(input: &mut &str) -> Result<Reference> {
    // Order is important for correct matching!
    alt((chapter, conference, article, web, book)).parse_next(input)
}

/// Parse an LNCS journal article reference
///
/// ```text
/// P. Smith, J. Doe, and A. Black. An example journal paper. International Journal of Examples, 12(3):45–67, 2019.
/// ```
pub fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Title
        preceded(separator, title_period_terminated),
        // Journal
        preceded(separator, take_while(1.., |c: char| c != ',')),
        // Volume
        preceded(separator, digit1),
        // Issue
        opt(preceded(
            opt(separator),
            delimited(("(", multispace0), digit1, (multispace0, ")")),
        )),
        // Pages
        opt(preceded((multispace0, ":", multispace0), pages)),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Date
        opt(preceded(separator, year_az)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(
                authors,
                title,
                journal,
                volume,
                issue,
                pages,
                doi_or_url,
                date_suffix,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Article),
                    id: Some(generate_id(&authors, &date_suffix)),
                    authors: Some(authors),
                    title: Some(title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(journal.trim())]),
                        volume_number: Some(IntegerOrString::from(volume)),
                        issue_number: issue.map(IntegerOrString::from),
                        ..Default::default()
                    })),
                    date: date_suffix.map(|(date, ..)| date),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an LNCS conference paper reference
///
/// ```text
/// P. Smith, J. Doe, and A. Black. An example conference paper. In Proceedings of the 10th International Conference on Examples, pages 123–135. Springer, 2020.
/// ```
pub fn conference(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Chapter Title
        preceded(separator, title_period_terminated),
        // "in" keyword
        preceded(separator, Caseless("In")),
        // Proceedings Title
        preceded(separator, take_while(1.., |c: char| c != ',')),
        // Pages
        opt(preceded((separator, Caseless("pages"), multispace0), pages)),
        // Publisher
        opt(preceded(separator, publisher_place)),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Year
        opt(preceded(separator, year_az)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(
                authors,
                chapter_title,
                _in,
                proceedings_title,
                pages,
                publisher,
                doi_or_url,
                date_suffix,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Article),
                    id: Some(generate_id(&authors, &date_suffix)),
                    authors: Some(authors),
                    title: Some(chapter_title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(proceedings_title.trim())]),
                        publisher,
                        ..Default::default()
                    })),
                    date: date_suffix.map(|(date, ..)| date),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an LNCS book chapter reference
///
/// ```text
/// P. Smith, J. Doe, and A. Black. Example chapter title. In A. Editor and B. Editor, editors, Title of the Book, pages 45–67. Springer, 2017.
/// ```
pub fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Chapter Title
        preceded(separator, title_period_terminated),
        // "in" keyword
        preceded(separator, Caseless("In")),
        // Editors
        delimited(
            separator,
            lncs_editors,
            (separator, Caseless("editor"), opt("s"), opt(",")),
        ),
        // Book Title
        preceded(separator, take_while(1.., |c: char| c != ',')),
        // Edition
        opt(preceded(separator, lncs_edition)),
        // Pages
        opt(preceded((separator, Caseless("pages"), multispace0), pages)),
        // Publisher
        opt(preceded(separator, publisher_place)),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Year
        opt(preceded(separator, year_az)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(
                authors,
                chapter_title,
                _in,
                editors,
                book_title,
                edition,
                pages,
                publisher,
                doi_or_url,
                date_suffix,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    id: Some(generate_id(&authors, &date_suffix)),
                    authors: Some(authors),
                    title: Some(chapter_title),
                    is_part_of: Some(Box::new(Reference {
                        title: Some(vec![t(book_title.trim())]),
                        editors: Some(editors),
                        version: edition,
                        publisher,
                        ..Default::default()
                    })),
                    date: date_suffix.map(|(date, ..)| date),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an LNCS book reference
///
/// ```text
/// P. Smith and J. Doe. An Example Book. Springer, Berlin, 2018.
/// ```
pub fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Title
        preceded(separator, title_period_terminated),
        // Edition: Optional edition (1st ed., 2nd ed., etc.)
        opt(preceded(separator, lncs_edition)),
        // Publisher: Parse place and publisher
        opt(preceded(separator, publisher_place)),
        // DOI or URL
        opt(preceded(separator, doi_or_url)),
        // Year: Publication year
        preceded(separator, year_az),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, title, edition, publisher, doi_or_url, (date, suffix), _terminator)| {
                Reference {
                    work_type: Some(CreativeWorkType::Book),
                    id: Some(generate_id(&authors, &Some((date.clone(), suffix)))),
                    authors: Some(authors),
                    title: Some(title),
                    version: edition,
                    publisher,
                    date: Some(date),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..Default::default()
                }
            },
        )
        .parse_next(input)
}

/// Parse an LNCS web resource reference
///
/// ```text
/// J. Doe. Title of the web page. http://www.example.com/, last accessed 10 Aug 2025.
/// ```
pub fn web(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        alt((person_given_family, organization)),
        // Title
        preceded(separator, title_period_terminated),
        // URL
        preceded(separator, url),
        // Last assessed
        opt(preceded(
            (
                separator,
                Caseless("last"),
                opt((multispace1, Caseless("accessed"))),
                multispace0,
                opt(":"),
                multispace0,
            ),
            take_while(1.., |c: char| c != '.' && c != ';'),
        )),
        // Optional terminator
        opt(terminator),
    )
        .map(|(author, title, url, _date, _terminator)| Reference {
            work_type: Some(CreativeWorkType::WebPage),
            id: Some(generate_id(slice::from_ref(&author), &None)),
            authors: Some(vec![author]),
            title: Some(title),
            url: Some(url),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse editors in LNCS formatting
fn lncs_editors(input: &mut &str) -> Result<Vec<Person>> {
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

/// Parse edition in LNCS format (1st edn., 2nd edn., etc.)
fn lncs_edition(input: &mut &str) -> Result<StringOrNumber> {
    terminated(
        (
            digit1,
            alt(("st", "nd", "rd", "th")),
            multispace1,
            Caseless("ed"),
            opt("n"),
        )
            .take(),
        opt("."),
    )
    .map(StringOrNumber::from)
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

    // These tests call the top level `lncs` parser to test for discrimination
    // between different work types.
    //
    // Avoid temptation to assert parsed details of works, instead relying on
    // the unit test for sub-parsers in other modules for that, where they exist.

    #[test]
    fn test_article() -> Result<()> {
        let reference = lncs(
            &mut r#"P. Smith, J. Doe, and A. Black. An example journal paper. International Journal of Examples, 12(3):45–67, 2019."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(3));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("An example journal paper".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|journal| journal.title.as_ref())
                .map(to_text),
            Some("International Journal of Examples".to_string())
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|part_of| part_of.volume_number.as_ref())
                .cloned(),
            Some(IntegerOrString::Integer(12))
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|part_of| part_of.issue_number.as_ref())
                .cloned(),
            Some(IntegerOrString::Integer(3))
        );
        assert_eq!(reference.page_start, Some(IntegerOrString::Integer(45)));
        assert_eq!(reference.page_end, Some(IntegerOrString::Integer(67)));
        assert!(reference.date.is_some());

        Ok(())
    }

    #[test]
    fn test_conference() -> Result<()> {
        let reference = lncs(
            &mut r#"P. Smith, J. Doe, and A. Black. An example conference paper. In Proceedings of the 10th International Conference on Examples, pages 123–135. Springer, 2020."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|proceedings| proceedings.editors.clone()),
            None
        );
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|proceedings| proceedings.version.clone()),
            None
        );
        assert!(reference.date.is_some());

        let reference = lncs(
            &mut r#"Vaishak Belle, Andrea Passerini, Guy Van den Broeck, et al. Probabilistic inference in hybrid domains by weighted model integration. In Proceedings of 24th International Joint Conference on Artificial Intelligence (IJCAI), pages 2770–2776. AAAI Press/International Joint Conferences on Artificial Intelligence, 2015."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|proceedings| proceedings.title.as_ref())
                .map(to_text),
            Some("Proceedings of 24th International Joint Conference on Artificial Intelligence (IJCAI)".to_string())
        );
        assert!(reference.date.is_some());

        Ok(())
    }

    #[test]
    fn test_chapter() -> Result<()> {
        let reference = lncs(
            &mut r#"P. Smith, J. Doe, and A. Black. Example chapter title. In A. Editor and B. Editor, editors, Title of the Book, 3rd edn. pages 45–67. Springer, 2017."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert!(reference.authors.is_some());
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(3));
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
                .and_then(|book| book.version.clone()),
            Some(StringOrNumber::String("3rd edn".into()))
        );
        assert!(reference.page_start.is_some());
        assert!(reference.page_end.is_some());
        assert_eq!(
            reference
                .is_part_of
                .as_ref()
                .and_then(|book| book.publisher.clone()),
            Some(PersonOrOrganization::Organization(Organization {
                name: Some("Springer".into()),
                ..Default::default()
            }))
        );
        assert!(reference.date.is_some());

        Ok(())
    }

    #[test]
    fn test_book() -> Result<()> {
        let reference =
            lncs(&mut r#"P. Smith and J. Doe. An Example Book. Springer, Berlin, 2018."#)?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("An Example Book".to_string())
        );
        assert_eq!(reference.is_part_of, None);
        assert_eq!(
            reference.publisher,
            Some(PersonOrOrganization::Organization(Organization {
                name: Some("Springer".into()),
                options: Box::new(OrganizationOptions {
                    address: Some(PostalAddressOrString::String("Berlin".into())),
                    ..Default::default()
                }),
                ..Default::default()
            }))
        );
        assert_eq!(reference.doi, None);
        assert_eq!(reference.url, None);
        assert!(reference.date.is_some());

        Ok(())
    }

    #[test]
    fn test_web() -> Result<()> {
        let reference = lncs(
            &mut r#"J. Doe. Title of the web page. http://www.example.com/, last accessed 10 Aug 2025."#,
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(1));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Title of the web page".to_string())
        );
        assert_eq!(reference.date, None);

        Ok(())
    }
}
