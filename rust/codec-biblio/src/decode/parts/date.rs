//! Parsers that parse a Stencila [`Date`] from a string

use winnow::{
    Parser, Result,
    ascii::digit1,
    combinator::{not, opt, peek, terminated},
    token::take_while,
};

use stencila_codec::stencila_schema::Date;

/// Parse a 4 digit year in range 1200-2050
pub fn year(input: &mut &str) -> Result<Date> {
    terminated(
        take_while(4..=4, |c: char| c.is_ascii_digit()),
        not(peek(digit1)),
    )
    .verify(|year: &str| {
        year.parse()
            .map_or_else(|_| false, |year: u32| (1200..=2050).contains(&year))
    })
    .map(|year: &str| Date {
        value: year.into(),
        ..Default::default()
    })
    .parse_next(input)
}

/// Parse a 4 digit year with an optional single suffix a-z (e.g 2010a)
pub fn year_az(input: &mut &str) -> Result<(Date, Option<String>)> {
    (
        year,
        opt(take_while(1..=1, |c: char| c.is_ascii_lowercase()).map(String::from)),
    )
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_year() -> Result<()> {
        let date = year(&mut "1200")?;
        assert_eq!(date.year(), Some(1200));

        let date = year(&mut "2050")?;
        assert_eq!(date.year(), Some(2050));

        assert!(year(&mut "123").is_err());
        assert!(year(&mut "1199").is_err());
        assert!(year(&mut "2051").is_err());
        assert!(year(&mut "12345").is_err());

        Ok(())
    }

    #[test]
    fn test_year_az() -> Result<()> {
        let (date, suffix) = year_az(&mut "2023")?;
        assert_eq!(date.year(), Some(2023));
        assert_eq!(suffix, None);

        let (date, suffix) = year_az(&mut "2023a")?;
        assert_eq!(date.year(), Some(2023));
        assert_eq!(suffix, Some("a".to_string()));

        let (date, suffix) = year_az(&mut "2023z")?;
        assert_eq!(date.year(), Some(2023));
        assert_eq!(suffix, Some("z".to_string()));

        Ok(())
    }
}
