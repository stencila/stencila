//! Parsers that parse a Stencila [`Date`] from a string

use winnow::{
    Parser, Result,
    ascii::digit1,
    combinator::{not, peek, terminated},
    token::take_while,
};

use codec::schema::Date;

/// Parse a 4 digit year
pub fn year(input: &mut &str) -> Result<Date> {
    terminated(
        take_while(4..=4, |c: char| c.is_ascii_digit()),
        not(peek(digit1)),
    )
    .map(|year: &str| Date {
        value: year.into(),
        ..Default::default()
    })
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_year() -> Result<()> {
        let date = year(&mut "1000")?;
        assert_eq!(date.year(), Some(1000));

        let date = year(&mut "9999")?;
        assert_eq!(date.year(), Some(9999));

        assert!(year(&mut "123").is_err());
        assert!(year(&mut "12345").is_err());

        Ok(())
    }
}
