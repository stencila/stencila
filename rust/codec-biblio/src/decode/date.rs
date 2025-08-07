//! Parsers that parse a Stencila [`Date`] from a string

use winnow::{Parser, Result, token::take_while};

use codec::schema::Date;

/// Parse a 4 digit year
pub fn year(input: &mut &str) -> Result<Date> {
    take_while(4..=4, |c: char| c.is_ascii_digit())
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

        Ok(())
    }
}
