//! Fallback parsers for extracting bibliographic information from unstructured text
//!
//! This module provides fallback parsing capabilities when structured citation parsers
//! (ie. APA, MLA, Chicago etc) fail to parse a reference. The fallback parsers
//! attempt to extract useful bibliographic information in the following priority order:
//!
//! 1. **DOI extraction**: Find DOI anywhere in text and use surrounding text
//! 2. **URL extraction**: Find non-DOI URLs and use surrounding text  
//! 3. **Plain text**: Use entire text (last resort)
//!
//! The key feature is handling DOIs and URLs that are nested within surrounding text,
//! not just at the beginning or end of the input.

use winnow::{
    Parser,
    ascii::{alphanumeric1, multispace0, multispace1},
    combinator::{alt, not, opt, peek, repeat, terminated},
    token::any,
};

use stencila_codec::stencila_schema::{
    Author, CreativeWorkType, Date, Reference, ReferenceOptions,
};

use crate::decode::{
    parts::{authors::persons, date::year_az, doi::doi_or_url},
    reference::generate_id,
};

enum Part {
    Date((Date, Option<String>)),
    Doi(String),
    Url(String),
    None,
}

/// Main fallback parser for unstructured bibliographic text
pub fn fallback(mut input: &str) -> Reference {
    (
        // Use `persons`, rather than `authors`, here because the latter includes organizations
        // which is very permissive and creates unwarranted matches.
        opt(terminated(
            persons.map(|persons| persons.into_iter().map(Author::Person).collect()),
            alt((
                (multispace0, alt((",", ".", ";")), multispace0).take(),
                multispace1,
            )),
        )),
        repeat(
            0..,
            alt((
                terminated(year_az, peek(not(alphanumeric1))).map(Part::Date),
                doi_or_url.map(|doi_or_url| {
                    if let Some(doi) = doi_or_url.doi {
                        Part::Doi(doi)
                    } else if let Some(url) = doi_or_url.url {
                        Part::Url(url)
                    } else {
                        Part::None
                    }
                }),
                any.map(|_| Part::None),
            )),
        ),
    )
        .map(|(authors, parts): (_, Vec<_>)| -> Reference {
            let mut reference = Reference {
                authors,
                ..Default::default()
            };
            let mut reference_date_with_suffix = None;

            for part in parts {
                match part {
                    Part::Date((date, date_suffix)) => {
                        if reference.date.is_none() {
                            reference.date = Some(date.clone());
                            reference_date_with_suffix = Some((date, date_suffix));
                        }
                    }
                    Part::Doi(doi) => {
                        if reference.doi.is_none() {
                            reference.doi = Some(doi);
                        }
                    }
                    Part::Url(url) => {
                        if reference.url.is_none() {
                            reference.work_type = Some(CreativeWorkType::WebPage);
                            reference.url = Some(url)
                        }
                    }
                    Part::None => {}
                }
            }

            if let Some(authors) = &reference.authors {
                reference.id = Some(generate_id(authors, &reference_date_with_suffix));
            }

            reference.options.text = some_if_not_blank(input);

            reference
        })
        .parse_next(&mut input)
        .unwrap_or_else(|_| Reference {
            options: Box::new(ReferenceOptions {
                text: some_if_not_blank(input),
                ..Default::default()
            }),
            ..Default::default()
        })
}

fn some_if_not_blank(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then_some(text.to_string())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_extract_authors_date() {
        let r = fallback("Smith, J & Jones, P (1991) Some title. 10.1234/example");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, Some("smith-and-jones-1991".to_string()));
        assert_eq!(r.authors.iter().flatten().count(), 2);
        assert_eq!(r.date, Some(Date::new("1991".into())));
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(r.url, None);
    }

    #[test]
    fn test_extract_doi() {
        let r = fallback("10.1234/example");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(r.url, None);

        let r = fallback("10.1234/example Research paper about climate change");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(r.url, None);

        let r = fallback("Research paper about climate change doi:10.1234/example");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(r.url, None);

        let r = fallback("Climate research (10.1234/example) shows warming trends");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(r.url, None);

        let r = fallback("Study on AI https://doi.org/10.1234/example from 2023");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, Some(Date::new("2023".into())));
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(r.url, None);
    }

    #[test]
    fn test_extract_url() {
        let r = fallback("Web resource about programming https://example.com/guide tutorial");
        assert_eq!(r.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, None);
        assert_eq!(r.url, Some("https://example.com/guide".to_string()));

        let r = fallback("Plain text with a url https://example.org");
        assert_eq!(r.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, None);
        assert_eq!(r.url, Some("https://example.org".to_string()));
    }

    #[test]
    fn test_plain_text() {
        let r = fallback("Some unstructured reference text about research");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, None);
        assert_eq!(r.url, None);
        assert_eq!(
            r.options.text,
            Some("Some unstructured reference text about research".to_string())
        );
    }

    #[test]
    fn test_empty_input() {
        let r = fallback("");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, None);
        assert_eq!(r.url, None);
        assert_eq!(r.options.text, None);

        let r = fallback("   \t\n   ");
        assert_eq!(r.work_type, None);
        assert_eq!(r.id, None);
        assert_eq!(r.authors, None);
        assert_eq!(r.date, None);
        assert_eq!(r.doi, None);
        assert_eq!(r.url, None);
        assert_eq!(r.options.text, None);
    }
}
