//! Parsers that parse a Stencila `doi` property from strings
//!
//! This module provides parsers for extracting DOI (Digital Object Identifier)
//! information from various formats including:
//! - Bare DOIs: "10.1234/example.123"
//! - Prefixed DOIs: "doi:10.1234/example.123", "DOI:10.1234/example.123"
//! - DOI URLs: "https://doi.org/10.1234/example.123", "http://dx.doi.org/10.1234/example.123"

use winnow::{
    Parser, Result,
    ascii::{Caseless, multispace0},
    combinator::{alt, opt, preceded},
    token::take_while,
};

use crate::decode::parts::url::url;

/// Parse DOI information from a string
///
/// This parser tries to extract DOI information from various formats and returns
/// a [`Reference`] struct with the doi field populated. The parser attempts to match:
///
/// 1. DOI URLs (https://doi.org/, http://dx.doi.org/, etc.)
/// 2. Prefixed DOIs (doi:, DOI:, etc.)
/// 3. Bare DOIs (starting with 10.)
pub fn doi(input: &mut &str) -> Result<String> {
    alt((doi_url, doi_prefixed, doi_bare))
        .map(|doi| doi.trim_end_matches(['.', ',', ';']).to_string())
        .parse_next(input)
}

#[derive(Clone)]
pub struct DoiOrUrl {
    pub doi: Option<String>,
    pub url: Option<String>,
}

/// Get a DOI or URL
pub fn doi_or_url(input: &mut &str) -> Result<DoiOrUrl> {
    alt((doi, url))
        .map(|id| {
            if id.starts_with("10.") {
                DoiOrUrl {
                    doi: Some(id),
                    url: None,
                }
            } else {
                DoiOrUrl {
                    doi: None,
                    url: Some(id),
                }
            }
        })
        .parse_next(input)
}

/// Parse DOI URLs
///
/// Recognizes URLs like "https://doi.org/10.1234/example" or "http://dx.doi.org/10.1234/example"
fn doi_url<'s>(input: &mut &'s str) -> Result<&'s str> {
    preceded(
        alt((
            "https://doi.org/",
            "http://doi.org/",
            "https://dx.doi.org/",
            "http://dx.doi.org/",
            "https://www.doi.org/",
            "http://www.doi.org/",
        )),
        doi_bare,
    )
    .parse_next(input)
}

/// Parse prefixed DOIs
///
/// Recognizes DOIs with prefixes like "doi:10.1234/example" or "DOI:10.1234/example"
fn doi_prefixed<'s>(input: &mut &'s str) -> Result<&'s str> {
    preceded(
        (Caseless("doi"), multispace0, opt(":"), multispace0),
        doi_bare,
    )
    .parse_next(input)
}

/// Parse bare DOIs
///
/// Matches DOI strings that start with "10." followed by the registrant and suffix
fn doi_bare<'s>(input: &mut &'s str) -> Result<&'s str> {
    (
        "10.",
        take_while(4.., |c: char| c.is_numeric()),
        "/",
        alt((
            // Allow for balanced parentheses in DOIs
            (
                take_while(1.., |c: char| is_valid_doi_char(c)),
                "(",
                take_while(1.., |c: char| is_valid_doi_char(c)),
                ")",
                opt(take_while(1.., |c: char| is_valid_doi_char(c))),
            )
                .take(),
            take_while(1.., |c: char| is_valid_doi_char(c)),
        )),
    )
        .take()
        .parse_next(input)
}

/// Check if a character is valid in a DOI
///
/// DOIs can contain alphanumeric characters and various punctuation marks
fn is_valid_doi_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ';' | '/' | ':')
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_doi_bare() -> Result<()> {
        // Standard DOI format
        let d = doi(&mut "10.1234/example.123")?;
        assert_eq!(d, "10.1234/example.123");

        // DOI with complex suffix
        let d = doi(&mut "10.1038/nature12373")?;
        assert_eq!(d, "10.1038/nature12373");

        // DOI with punctuation in suffix
        let d = doi(&mut "10.1234/example-test_123.456")?;
        assert_eq!(d, "10.1234/example-test_123.456");

        Ok(())
    }

    #[test]
    fn test_doi_prefixed() -> Result<()> {
        // Lowercase prefix
        let d = doi(&mut "doi:10.1234/example.123")?;
        assert_eq!(d, "10.1234/example.123");

        // Uppercase prefix
        let d = doi(&mut "DOI:10.1038/nature12373")?;
        assert_eq!(d, "10.1038/nature12373");

        // Mixed case prefix
        let d = doi(&mut "Doi:10.1234/test")?;
        assert_eq!(d, "10.1234/test");

        // With space after colon
        let d = doi(&mut "doi: 10.1234/example")?;
        assert_eq!(d, "10.1234/example");

        Ok(())
    }

    #[test]
    fn test_doi_url() -> Result<()> {
        // HTTPS doi.org
        let d = doi(&mut "https://doi.org/10.1234/example.123")?;
        assert_eq!(d, "10.1234/example.123");

        // HTTP doi.org
        let d = doi(&mut "http://doi.org/10.1038/nature12373")?;
        assert_eq!(d, "10.1038/nature12373");

        // dx.doi.org subdomain
        let d = doi(&mut "https://dx.doi.org/10.1234/test")?;
        assert_eq!(d, "10.1234/test");

        // www.doi.org subdomain
        let d = doi(&mut "https://www.doi.org/10.1234/example")?;
        assert_eq!(d, "10.1234/example");

        Ok(())
    }

    #[test]
    fn test_doi_complex_suffixes() -> Result<()> {
        // DOI with parentheses
        let d = doi(&mut "10.1234/example(2023)")?;
        assert_eq!(d, "10.1234/example(2023)");

        // DOI with colons
        let d = doi(&mut "10.1234/journal:volume:issue")?;
        assert_eq!(d, "10.1234/journal:volume:issue");

        // DOI with semicolons
        let d = doi(&mut "10.1234/example;part2")?;
        assert_eq!(d, "10.1234/example;part2");

        // Real-world complex DOI
        let d = doi(&mut "10.1371/journal.pone.0123456")?;
        assert_eq!(d, "10.1371/journal.pone.0123456");

        Ok(())
    }

    #[test]
    fn test_doi_registrant_variations() -> Result<()> {
        // 4-digit registrant (minimum)
        let d = doi(&mut "10.1000/example")?;
        assert_eq!(d, "10.1000/example");

        // 5-digit registrant
        let d = doi(&mut "10.12345/example")?;
        assert_eq!(d, "10.12345/example");

        // 9-digit registrant (maximum)
        let d = doi(&mut "10.123456789/example")?;
        assert_eq!(d, "10.123456789/example");

        Ok(())
    }

    #[test]
    fn test_doi_specific_parsers() -> Result<()> {
        // Test doi_bare directly
        let d = doi_bare(&mut "10.1234/example")?;
        assert_eq!(d, "10.1234/example");

        // Test doi_prefixed directly
        let d = doi_prefixed(&mut "doi:10.1234/example")?;
        assert_eq!(d, "10.1234/example");

        // Test doi_url directly
        let d = doi_url(&mut "https://doi.org/10.1234/example")?;
        assert_eq!(d, "10.1234/example");

        Ok(())
    }
}
