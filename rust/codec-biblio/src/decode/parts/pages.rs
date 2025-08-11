//! Parsers that parse Stencila `pageStart`, `pageEnd` and `pagination` properties from strings

use winnow::{
    Parser, Result,
    ascii::{digit1, multispace0},
    combinator::alt,
    token::take_while,
};

use codec::schema::{IntegerOrString, Reference};

/// Parse pagination information from a string
///
/// This parser tries to extract page information from various formats and returns
/// a [`Reference`] struct with the appropriate page fields populated. The parser
/// attempts to match in order:
///
/// 1. Page ranges (e.g., "1-10", "23–45")
/// 2. Single pages (e.g., "42", "7")
/// 3. General pagination strings (e.g., "xii", "A1-A10")
pub fn pages(input: &mut &str) -> Result<Reference> {
    alt((page_range, page_single, pagination)).parse_next(input)
}

/// Parse a page range with start and end pages
///
/// Recognizes numeric page ranges separated by various dash characters including
/// hyphen-minus, en dash, hyphen, non-breaking hyphen, figure dash, em dash,
/// horizontal bar, and minus sign. Whitespace around the dash is allowed.
pub fn page_range(input: &mut &str) -> Result<Reference> {
    (
        digit1,
        (
            multispace0,
            // Hyphen-minus, En dash, Hyphen, Non-breaking hyphen, Figure dash, Em dash, Horizontal bar, Minus sign
            alt(("-", "–", "‐", "-", "‒", "—", "―", "−")),
            multispace0,
        ),
        digit1,
    )
        .map(|(start, _, end): (&str, _, &str)| Reference {
            page_start: Some(IntegerOrString::from(start)),
            page_end: Some(IntegerOrString::from(end)),
            ..Default::default()
        })
        .parse_next(input)
}

/// Parse a single page number
///
/// Matches a sequence of digits and sets it as the page_start in the Reference.
pub fn page_single(input: &mut &str) -> Result<Reference> {
    digit1
        .map(|page| Reference {
            page_start: Some(IntegerOrString::from(page)),
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
    take_while(1.., |c: char| !c.is_ascii_punctuation())
        .map(|pagination: &str| Reference {
            pagination: Some(pagination.into()),
            ..Default::default()
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_page_range() -> Result<()> {
        // Basic hyphen-minus
        let Reference {
            page_start,
            page_end,
            ..
        } = pages(&mut "1-9")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(1)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(9)));

        // En dash
        let Reference {
            page_start,
            page_end,
            ..
        } = pages(&mut "12–34")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(12)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(34)));

        // With spaces around dash
        let Reference {
            page_start,
            page_end,
            ..
        } = pages(&mut "100 - 200")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(100)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(200)));

        // Em dash
        let Reference {
            page_start,
            page_end,
            ..
        } = pages(&mut "5—15")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(5)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(15)));

        // Minus sign
        let Reference {
            page_start,
            page_end,
            ..
        } = pages(&mut "7−17")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(7)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(17)));

        Ok(())
    }

    #[test]
    fn test_page_single() -> Result<()> {
        let Reference {
            page_start,
            page_end,
            pagination,
            ..
        } = pages(&mut "42")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(42)));
        assert_eq!(page_end, None);
        assert_eq!(pagination, None);

        let Reference { page_start, .. } = pages(&mut "1")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(1)));

        let Reference { page_start, .. } = pages(&mut "999")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(999)));

        Ok(())
    }

    #[test]
    fn test_pagination() -> Result<()> {
        // Roman numerals
        let Reference {
            page_start,
            page_end,
            pagination,
            ..
        } = pages(&mut "xii")?;
        assert_eq!(page_start, None);
        assert_eq!(page_end, None);
        assert_eq!(pagination, Some("xii".to_string()));

        // Alphanumeric
        let Reference { pagination, .. } = pages(&mut "A1")?;
        assert_eq!(pagination, Some("A1".to_string()));

        // Complex pagination
        let Reference { pagination, .. } = pages(&mut "S123")?;
        assert_eq!(pagination, Some("S123".to_string()));

        // Mixed case
        let Reference { pagination, .. } = pages(&mut "Appendix")?;
        assert_eq!(pagination, Some("Appendix".to_string()));

        Ok(())
    }

    #[test]
    fn test_page_range_specific_parsers() -> Result<()> {
        // Test page_range directly
        let Reference {
            page_start,
            page_end,
            ..
        } = page_range(&mut "25-50")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(25)));
        assert_eq!(page_end, Some(IntegerOrString::Integer(50)));

        // Test page_single directly
        let Reference { page_start, .. } = page_single(&mut "77")?;
        assert_eq!(page_start, Some(IntegerOrString::Integer(77)));

        // Test pagination directly
        let Reference { pagination, .. } = pagination(&mut "iv")?;
        assert_eq!(pagination, Some("iv".to_string()));

        Ok(())
    }
}
