use winnow::{
    Parser, Result,
    combinator::{alt, delimited},
    token::take_while,
};

use stencila_codec::stencila_schema::{Inline, shortcuts::t};

use crate::decode::parts::chars::{
    is_double_close_quote, is_single_close_quote, one_double_close_quote, one_double_open_quote,
    one_single_close_quote, one_single_open_quote,
};

/// Parse a title terminated by a period
pub fn title_period_terminated(input: &mut &str) -> Result<Vec<Inline>> {
    take_while(1.., |c: char| c != '.')
        .map(|title: &str| vec![t(title.trim_start().trim_end_matches([',', ';', ' ']))])
        .parse_next(input)
}

/// Parse a title terminated by a semicolon
pub fn title_semicolon_terminated(input: &mut &str) -> Result<Vec<Inline>> {
    take_while(1.., |c: char| c != ';')
        .map(|title: &str| vec![t(title.trim_start().trim_end_matches([',', '.', ' ']))])
        .parse_next(input)
}

/// Parse a title in double or single quotes
pub fn title_quoted(input: &mut &str) -> Result<Vec<Inline>> {
    alt((
        delimited(
            one_double_open_quote,
            take_while(1.., |c: char| !is_double_close_quote(c)),
            one_double_close_quote,
        ),
        delimited(
            one_single_open_quote,
            take_while(1.., |c: char| !is_single_close_quote(c)),
            one_single_close_quote,
        ),
    ))
    .map(|title: &str| vec![t(title.trim_start().trim_end_matches(['.', ',', ';', ' ']))])
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_chicago_quoted_title() -> Result<()> {
        assert_eq!(title_quoted(&mut r#""The title""#)?, vec![t("The title")]);

        assert_eq!(title_quoted(&mut r#""The title.""#)?, vec![t("The title")]);

        // Test with smart quotes
        assert_eq!(
            title_quoted(&mut "\u{201c}Smart quotes\u{201d}")?,
            vec![t("Smart quotes")]
        );

        Ok(())
    }
}
