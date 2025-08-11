use winnow::{
    Parser, Result,
    ascii::{Caseless, multispace0},
    combinator::{alt, not, opt, preceded},
    token::take_while,
};

/// Parse a URL
pub fn url(input: &mut &str) -> Result<String> {
    preceded(
        opt((Caseless("URL"), multispace0, opt(":"), multispace0)),
        (
            alt(("https://", "http://")),
            preceded(
                not(alt(("doi.org/", "dx.doi.org/", "www.doi.org/"))),
                take_while(1.., |c: char| !c.is_ascii_whitespace()),
            ),
        ),
    )
    .map(|(prefix, suffix): (&str, &str)| {
        [prefix, suffix.trim_end_matches(['.', ',', ';'])].concat()
    })
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_url() -> Result<()> {
        assert_eq!(url(&mut "https://example.com")?, "https://example.com");
        assert_eq!(url(&mut "http://example.com")?, "http://example.com");

        assert_eq!(url(&mut "URL https://example.com")?, "https://example.com");
        assert_eq!(
            url(&mut "urL https://example.com/some/path.html")?,
            "https://example.com/some/path.html"
        );
        assert_eq!(
            url(&mut "urL : https://example.com?a=1&b=2")?,
            "https://example.com?a=1&b=2"
        );

        assert_eq!(
            url(&mut "https://example.com/some/path.html.")?,
            "https://example.com/some/path.html"
        );

        Ok(())
    }
}
