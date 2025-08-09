//! Fallback parsers for extracting bibliographic information from unstructured text
//!
//! This module provides fallback parsing capabilities when structured citation parsers
//! (APA, MLA, Chicago, IEEE, Vancouver) fail to parse a reference. The fallback parsers
//! attempt to extract useful bibliographic information in the following priority order:
//!
//! 1. **DOI extraction**: Find DOI anywhere in text and use surrounding text as title
//! 2. **URL extraction**: Find non-DOI URLs and use surrounding text as title  
//! 3. **Plain text**: Use entire text as title (last resort)
//!
//! The key feature is handling DOIs and URLs that are nested within surrounding text,
//! not just at the beginning or end of the input.

use winnow::{
    Parser, Result,
    combinator::{alt, repeat, repeat_till},
    token::any,
};

use codec::schema::{Reference, shortcuts::t};

use crate::decode::doi::{DoiOrUrl, doi_or_url};

/// Main fallback parser for unstructured bibliographic text
pub fn fallback(input: &mut &str) -> Result<Reference> {
    alt((
        repeat_till(0.., any, doi_or_url).map(|(text, doi_or_url): (String, DoiOrUrl)| {
            let title = clean_title_text(&text);
            Reference {
                title: (!title.is_empty()).then_some(vec![t(title)]),
                doi: doi_or_url.doi,
                url: doi_or_url.url,
                ..Default::default()
            }
        }),
        repeat(0.., any).map(|text: String| {
            let title = clean_title_text(&text);
            Reference {
                title: (!title.is_empty()).then_some(vec![t(title)]),
                ..Default::default()
            }
        }),
    ))
    .parse_next(input)
}

/// Clean up text to make it suitable as a reference title
///
/// Removes common prefixes, suffixes, and cleans whitespace while
/// preserving meaningful content and punctuation.
fn clean_title_text(text: &str) -> String {
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
    use codec_text_trait::to_text;
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_extract_doi_at_beginning() -> Result<()> {
        let reference = fallback(&mut "10.1234/example Research paper about climate change")?;
        assert_eq!(reference.doi, Some("10.1234/example".to_string()));
        Ok(())
    }

    #[test]
    fn test_extract_doi_at_end() -> Result<()> {
        let reference = fallback(&mut "Research paper about climate change doi:10.1234/example")?;
        assert_eq!(reference.doi, Some("10.1234/example".to_string()));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Research paper about climate change".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_extract_doi_in_middle() -> Result<()> {
        let reference = fallback(&mut "Climate research (10.1234/example) shows warming trends")?;
        assert_eq!(reference.doi, Some("10.1234/example".to_string()));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Climate research".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_extract_doi_url() -> Result<()> {
        let reference = fallback(&mut "Study on AI https://doi.org/10.1234/example from 2023")?;
        assert_eq!(reference.doi, Some("10.1234/example".to_string()));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Study on AI".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_extract_doi_only() -> Result<()> {
        let reference = fallback(&mut "10.1234/example")?;
        assert_eq!(reference.doi, Some("10.1234/example".to_string()));
        assert!(reference.title.is_none());
        Ok(())
    }

    #[test]
    fn test_extract_url_with_context() -> Result<()> {
        let reference =
            fallback(&mut "Web resource about programming https://example.com/guide tutorial")?;
        assert_eq!(reference.url, Some("https://example.com/guide".to_string()));
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Web resource about programming".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_plain_text_fallback() -> Result<()> {
        let reference = fallback(&mut "Some unstructured reference text about research")?;
        assert!(reference.doi.is_none());
        assert!(reference.url.is_none());
        assert_eq!(
            reference.title.map(|title| to_text(&title)),
            Some("Some unstructured reference text about research".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_clean_title_text() {
        assert_eq!(clean_title_text("  Research paper  "), "Research paper");
        assert_eq!(clean_title_text("- Research paper -"), "Research paper");
        assert_eq!(clean_title_text(": Title text :"), "Title text");
        assert_eq!(
            clean_title_text("Title, with, commas."),
            "Title, with, commas"
        );
        assert_eq!(
            clean_title_text("  Multiple   spaces   normalized  "),
            "Multiple spaces normalized"
        );
        assert_eq!(clean_title_text(""), "");
    }

    #[test]
    fn test_empty_input() -> Result<()> {
        let reference = fallback(&mut "")?;
        assert!(reference.doi.is_none());
        assert!(reference.url.is_none());
        assert!(reference.title.is_none());
        Ok(())
    }

    #[test]
    fn test_whitespace_only() -> Result<()> {
        let reference = fallback(&mut "   \t\n   ")?;
        assert!(reference.doi.is_none());
        assert!(reference.url.is_none());
        assert!(reference.title.is_none());
        Ok(())
    }

    #[test]
    fn debug_doi_parser() -> Result<()> {
        use crate::decode::doi::doi;

        let result1 = doi.parse_peek("Climate");
        let result2 = doi.parse_peek("10.1234/example");

        eprintln!("DOI parser on 'Climate': {:?}", result1);
        eprintln!("DOI parser on '10.1234/example': {:?}", result2);

        Ok(())
    }
}
