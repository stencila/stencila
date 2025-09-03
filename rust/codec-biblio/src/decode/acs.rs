//! Parsers that parse Stencila [`Reference`] nodes from strings in ACS reference list format

use winnow::{
    Parser, Result,
    ascii::{Caseless, multispace0},
    combinator::{alt, delimited, opt, preceded, separated},
};

use stencila_codec::stencila_schema::{
    Author, CreativeWorkType, Organization, Person, PersonOptions, Reference, ReferenceOptions,
};

use crate::decode::{
    parts::{
        authors::{authors, organization, person_family_initials},
        date::year_az,
        doi::doi_or_url,
        is_part_of::{in_book, in_journal},
        journal::journal_no_comma,
        pages::pages,
        publisher::publisher_place,
        separator::separator,
        terminator::terminator,
        title::{title_period_terminated, title_semicolon_terminated},
        volume::{issue, volume},
    },
    reference::generate_id,
};

/// Parse a Stencila [`Reference`] from a ACS reference list item
#[allow(unused)]
pub fn acs(input: &mut &str) -> Result<Reference> {
    // Order is important for correct matching!
    alt((chapter, article, book)).parse_next(input)
}

/// Parse a ACS journal article reference
///
/// ```text
/// Author, A. B.; Author, C.D. Title of Article. J. Name Year, Volume (Issue), Pages.
/// ```
pub fn article(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Title
        preceded(separator, title_period_terminated),
        // Journal
        preceded(separator, journal_no_comma),
        // Year
        preceded(separator, year_az),
        // Volume
        opt(preceded(separator, volume)),
        // Issue
        opt(delimited(
            (multispace0, "(", multispace0),
            issue,
            (multispace0, ")", multispace0),
        )),
        // Pages
        opt(preceded(separator, pages)),
        // DOI or URL (optional)
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(
                authors,
                title,
                journal_title,
                (date, suffix),
                volume,
                issue,
                pages,
                doi_or_url,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Article),
                    id: Some(generate_id(&authors, &Some((date.clone(), suffix)))),
                    authors: Some(authors),
                    title: Some(title),
                    is_part_of: in_journal(journal_title, volume, issue),
                    date: Some(date),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse a ACS book reference
///
/// ```text
/// Author, A. B. Book Title; Publisher: Place, Year.
/// ```
pub fn book(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Title
        preceded(separator, title_semicolon_terminated),
        // Publisher: Place
        opt(preceded(separator, publisher_place)),
        // Year
        preceded(separator, year_az),
        // DOI or URL (optional)
        opt(preceded(separator, doi_or_url)),
        // Optional terminator
        opt(terminator),
    )
        .map(
            |(authors, title, publisher, (date, suffix), doi_or_url, _terminator)| Reference {
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

/// Parse a ACS book chapter reference
///
/// ```text
/// Author, A. B. Chapter Title. In Book Title; Editor, C. D., Ed.; Publisher: Place, Year; pp Pages.
/// ```
pub fn chapter(input: &mut &str) -> Result<Reference> {
    (
        // Authors
        authors,
        // Chapter Title
        preceded(separator, title_period_terminated),
        // "In" keyword
        preceded(separator, (Caseless("In"), opt(":"))),
        // Editors
        preceded(separator, acs_editors),
        // Book Title
        preceded(separator, title_semicolon_terminated),
        // Publisher: Place
        opt(preceded(separator, publisher_place)),
        // Year
        opt(preceded(separator, year_az)),
        // Pages
        opt(preceded(separator, pages)),
        // DOI or URL (optional)
        opt(preceded(separator, doi_or_url)),
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
                publisher,
                date_suffix,
                pages,
                doi_or_url,
                _terminator,
            )| {
                Reference {
                    work_type: Some(CreativeWorkType::Chapter),
                    id: Some(generate_id(&authors, &date_suffix)),
                    authors: Some(authors),
                    title: Some(chapter_title),
                    date: date_suffix.map(|(date, ..)| date),
                    is_part_of: in_book(book_title, Some(editors), publisher, None),
                    doi: doi_or_url.clone().and_then(|doi_or_url| doi_or_url.doi),
                    url: doi_or_url.and_then(|doi_or_url| doi_or_url.url),
                    ..pages.unwrap_or_default()
                }
            },
        )
        .parse_next(input)
}

/// Parse editors from ACS references
fn acs_editors(input: &mut &str) -> Result<Vec<Person>> {
    separated(
        1..,
        alt((person_family_initials, organization)).map(|author| match author {
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
    use stencila_codec::stencila_schema::IntegerOrString;
    use stencila_codec_text_trait::to_text;

    use super::*;

    #[test]
    fn test_article() -> Result<()> {
        let reference = acs(
            &mut "Buttenschoen, M.; Morris, G. M.; Deane, C. M. PoseBusters: AI-Based Docking Methods Fail to Generate Physically Valid Poses or Generalise to Novel Sequences. Chem. Sci. 2024, 15 (9), 3130-3139",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            reference
                .is_part_of
                .clone()
                .and_then(|part_of| part_of.options.volume_number),
            Some(IntegerOrString::Integer(15))
        );
        assert_eq!(
            reference
                .is_part_of
                .clone()
                .and_then(|part_of| part_of.options.issue_number),
            Some(IntegerOrString::Integer(9))
        );
        assert_eq!(
            reference.options.page_end,
            Some(IntegerOrString::Integer(3139))
        );

        Ok(())
    }

    #[test]
    fn test_book() -> Result<()> {
        // Basic ACS book format with place and publisher
        let reference =
            acs(&mut "Smith J.; Jones A. Programming Guide; Tech Press: New York, 2023.")?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Book));
        assert!(reference.authors.is_some());
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Programming Guide".to_string())
        );
        assert!(reference.options.publisher.is_some());
        assert!(reference.date.is_some());

        Ok(())
    }

    #[test]
    fn test_chapter() -> Result<()> {
        // Basic ACS chapter with multiple chapter authors and single editor
        let reference = acs(
            &mut "Smith J, Brown K. Research methods. In: Jones B. Handbook of Psychology; Academic Press: New York, 2020. p. 15-30.",
        )?;
        assert_eq!(reference.work_type, Some(CreativeWorkType::Chapter));
        assert!(reference.authors.is_some());
        assert_eq!(reference.authors.map(|authors| authors.len()), Some(2));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Research methods".to_string())
        );
        assert!(reference.is_part_of.is_some());
        if let Some(book) = reference.is_part_of.as_ref() {
            assert_eq!(
                book.title.as_ref().map(to_text),
                Some("Handbook of Psychology".to_string())
            );
            assert!(book.options.editors.is_some());
            assert_eq!(
                book.options.editors.as_ref().map(|editors| editors.len()),
                Some(1)
            );
            assert!(book.options.publisher.is_some());
        }
        assert!(reference.date.is_some());
        assert!(reference.options.page_start.is_some());
        assert!(reference.options.page_end.is_some());

        Ok(())
    }
}
