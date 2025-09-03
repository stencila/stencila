//! Parsers that parse Stencila `pageStart`, `pageEnd` and `pagination` properties from strings

use winnow::{
    Parser, Result,
    ascii::multispace0,
    combinator::{alt, opt, preceded},
    token::take_while,
};

use stencila_codec::stencila_schema::{IntegerOrString, Reference, ReferenceOptions};

use crate::decode::parts::chars::is_hyphen;

/// Parse pagination information from a string
///
/// This parser tries to extract page information from various formats and returns
/// a [`Reference`] struct with the appropriate page fields populated. The parser
/// attempts to match in order:
///
/// 1. Page ranges (e.g., "1-10", "pp. 23–45")
/// 2. Single pages (e.g., "42", "p. 7")
/// 3. General pagination strings (e.g., "xii", "A1-A10")
pub fn pages(input: &mut &str) -> Result<Reference> {
    alt((page_range, page_single, pagination)).parse_next(input)
}

/// Parse a page range with start and end pages
///
/// Recognizes numeric page ranges separated by one or two hyphens.
/// Whitespace around the dash is allowed.
pub fn page_range(input: &mut &str) -> Result<Reference> {
    preceded(
        (opt("p"), opt("p"), opt("."), multispace0),
        (
            take_while(1.., |c: char| c.is_alphanumeric()),
            (multispace0, take_while(1..=2, is_hyphen), multispace0),
            take_while(1.., |c: char| c.is_alphanumeric()),
        ),
    )
    .map(|(start, _, end): (&str, _, &str)| Reference {
        options: Box::new(ReferenceOptions {
            page_start: Some(IntegerOrString::from(start)),
            page_end: Some(IntegerOrString::from(end)),
            ..Default::default()
        }),
        ..Default::default()
    })
    .parse_next(input)
}

/// Parse a single page number
///
/// Matches a sequence of digits and sets it as the page_start in the Reference.
pub fn page_single(input: &mut &str) -> Result<Reference> {
    preceded(
        (opt("p"), opt("."), multispace0),
        take_while(1.., |c: char| c.is_alphanumeric()),
    )
    .map(|page| Reference {
        options: Box::new(ReferenceOptions {
            page_start: Some(IntegerOrString::from(page)),
            ..Default::default()
        }),
        ..Default::default()
    })
    .parse_next(input)
}

/// Parse general pagination strings
///
/// Fallback parser for non-numeric pagination like Roman numerals, alphanumeric
/// sequences, or other complex page identifiers. Accepts any sequence of
/// non-punctuation characters.
pub fn pagination(input: &mut &str) -> Result<Reference> {
    preceded(
        (opt("p"), opt("p"), opt("."), multispace0),
        take_while(1.., |c: char| c.is_alphanumeric()),
    )
    .map(|pagination: &str| Reference {
        options: Box::new(ReferenceOptions {
            pagination: Some(pagination.into()),
            ..Default::default()
        }),
        ..Default::default()
    })
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn pages_(
        input: &str,
    ) -> Result<(
        Option<IntegerOrString>,
        Option<IntegerOrString>,
        Option<String>,
    )> {
        let mut input = input;
        let Reference { options, .. } = pages(&mut input)?;
        Ok((options.page_start, options.page_end, options.pagination))
    }

    #[test]
    fn test_page_range() -> Result<()> {
        // Basic hyphen-minus
        let (page_start, page_end, ..) = pages_("1-9")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(1)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(9)));

        // En dash
        let (page_start, page_end, ..) = pages_("12–34")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(12)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(34)));

        // With spaces around dash
        let (page_start, page_end, ..) = pages_("100 - 200")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(100)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(200)));

        // Em dash
        let (page_start, page_end, ..) = pages_("5—15")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(5)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(15)));

        // Minus sign
        let (page_start, page_end, ..) = pages_("7−17")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(7)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(17)));

        Ok(())
    }

    #[test]
    fn test_page_single() -> Result<()> {
        let (page_start, page_end, pagination) = pages_("42")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(42)));
        assert_eq!(page_end, None);
        assert_eq!(pagination, None);

        let (page_start, ..) = pages_("1")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(1)));

        let (page_start, ..) = pages_("999")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(999)));

        Ok(())
    }

    #[test]
    fn test_non_numeric() -> Result<()> {
        // Roman numerals
        let (page_start, page_end, pagination) = pages_("xii")?;
        assert_eq!(page_start, Some(IntegerOrString::from("xii")));
        assert_eq!(page_end, None);
        assert_eq!(pagination, None);

        // Alphanumeric
        let (page_start, ..) = pages_("A1")?;
        assert_eq!(page_start, Some(IntegerOrString::from("A1")));

        // Complex pagination
        let (page_start, ..) = pages_("S123")?;
        assert_eq!(page_start, Some(IntegerOrString::from("S123")));

        // Mixed case
        let (page_start, ..) = pages_("Appendix")?;
        assert_eq!(page_start, Some(IntegerOrString::from("Appendix")));

        Ok(())
    }

    #[test]
    fn test_page_range_specific_parsers() -> Result<()> {
        // Test page_range directly
        let reference = page_range(&mut "25-50")?;
        assert_eq!(
            reference.options.page_start,
            Some(IntegerOrString::Integer(25))
        );
        assert_eq!(
            reference.options.page_end,
            Some(IntegerOrString::Integer(50))
        );

        // Test page_single directly
        let reference = page_single(&mut "77")?;
        assert_eq!(
            reference.options.page_start,
            Some(IntegerOrString::Integer(77))
        );

        // Test pagination directly
        let reference = pagination(&mut "iv")?;
        assert_eq!(reference.options.pagination, Some("iv".to_string()));

        Ok(())
    }
}
