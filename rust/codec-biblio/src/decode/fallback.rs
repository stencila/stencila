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
    Parser, Result,
    combinator::{alt, repeat, repeat_till},
    token::any,
};

use codec::schema::{CreativeWorkType, Reference};

use crate::decode::parts::doi::{DoiOrUrl, doi_or_url};

/// Main fallback parser for unstructured bibliographic text
pub fn fallback(input: &mut &str) -> Result<Reference> {
    alt((
        repeat_till(0.., any, doi_or_url).map(|(text, doi_or_url): (String, DoiOrUrl)| {
            let text = clean_text(&text);
            let text = (!text.is_empty()).then_some(text);

            if doi_or_url.url.is_some() {
                Reference {
                    work_type: Some(CreativeWorkType::WebPage),
                    text,
                    url: doi_or_url.url,
                    ..Default::default()
                }
            } else {
                Reference {
                    text,
                    doi: doi_or_url.doi,
                    ..Default::default()
                }
            }
        }),
        repeat(0.., any).map(|text: String| {
            let text = clean_text(&text);
            let text = (!text.is_empty()).then_some(text);

            Reference {
                text,
                ..Default::default()
            }
        }),
    ))
    .parse_next(input)
}

/// Clean up text to make it suitable as a reference text
///
/// Removes common prefixes, suffixes, and cleans whitespace while
/// preserving meaningful content and punctuation.
fn clean_text(text: &str) -> String {
    text.trim()
        // Remove common separators at start/end
        .trim_start_matches(['-', '–', '—', ':', ';', ',', '.'])
        .trim_end_matches(['-', '–', '—', ':', ';', ',', '.', '('])
        // Normalize multiple spaces to single spaces
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_extract_doi() -> Result<()> {
        let r = fallback(&mut "10.1234/example")?;
        assert_eq!(r.work_type, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert!(r.text.is_none());

        let r = fallback(&mut "10.1234/example Research paper about climate change")?;
        assert_eq!(r.work_type, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));

        let r = fallback(&mut "Research paper about climate change doi:10.1234/example")?;
        assert_eq!(r.work_type, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(
            r.text,
            Some("Research paper about climate change".to_string())
        );

        let r = fallback(&mut "Climate research (10.1234/example) shows warming trends")?;
        assert_eq!(r.work_type, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(r.text, Some("Climate research".to_string()));

        let r = fallback(&mut "Study on AI https://doi.org/10.1234/example from 2023")?;
        assert_eq!(r.work_type, None);
        assert_eq!(r.doi, Some("10.1234/example".to_string()));
        assert_eq!(r.text, Some("Study on AI".to_string()));

        Ok(())
    }

    #[test]
    fn test_extract_url() -> Result<()> {
        let r = fallback(&mut "Web resource about programming https://example.com/guide tutorial")?;
        assert_eq!(r.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(r.url, Some("https://example.com/guide".to_string()));
        assert_eq!(r.text, Some("Web resource about programming".to_string()));

        let r = fallback(&mut "Plain text with a url https://example.org")?;
        assert_eq!(r.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(r.url, Some("https://example.org".to_string()));
        assert_eq!(r.text, Some("Plain text with a".to_string()));

        Ok(())
    }

    #[test]
    fn test_plain_text() -> Result<()> {
        let r = fallback(&mut "Some unstructured reference text about research")?;
        assert!(r.doi.is_none());
        assert!(r.url.is_none());
        assert_eq!(
            r.text,
            Some("Some unstructured reference text about research".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_clean_text() {
        assert_eq!(clean_text("  Research paper  "), "Research paper");
        assert_eq!(clean_text("- Research paper -"), "Research paper");
        assert_eq!(clean_text(": Title text :"), "Title text");
        assert_eq!(clean_text("Title, with, commas."), "Title, with, commas");
        assert_eq!(
            clean_text("  Multiple   spaces   normalized  "),
            "Multiple spaces normalized"
        );
        assert_eq!(clean_text(""), "");
    }

    #[test]
    fn test_empty_input() -> Result<()> {
        let r = fallback(&mut "")?;
        assert!(r.doi.is_none());
        assert!(r.url.is_none());
        assert!(r.text.is_none());

        let r = fallback(&mut "   \t\n   ")?;
        assert!(r.doi.is_none());
        assert!(r.url.is_none());
        assert!(r.text.is_none());

        Ok(())
    }
}
