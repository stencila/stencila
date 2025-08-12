use winnow::{
    Parser, Result,
    ascii::{Caseless, multispace0},
    combinator::{opt, preceded},
    token::take_while,
};

use codec::schema::IntegerOrString;

/// Parse volume number with "vol." prefix
pub fn vol_prefixed_volume(input: &mut &str) -> Result<IntegerOrString> {
    preceded(
        (Caseless("vol"), multispace0, opt("."), multispace0),
        take_while(1.., |c: char| c.is_alphanumeric()),
    )
    .map(IntegerOrString::from)
    .parse_next(input)
}

/// Parse issue number with "no." prefix  
pub fn no_prefixed_issue(input: &mut &str) -> Result<IntegerOrString> {
    preceded(
        (Caseless("no"), multispace0, opt("."), multispace0),
        take_while(1.., |c: char| c.is_alphanumeric()),
    )
    .map(IntegerOrString::from)
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume() -> Result<()> {
        assert_eq!(vol_prefixed_volume(&mut "vol. 1")?, IntegerOrString::Integer(1));
        assert_eq!(vol_prefixed_volume(&mut "vol . 123")?, IntegerOrString::Integer(123));
        assert_eq!(vol_prefixed_volume(&mut "VOL 456")?, IntegerOrString::Integer(456));

        Ok(())
    }

    #[test]
    fn test_issue() -> Result<()> {
        assert_eq!(no_prefixed_issue(&mut "no. 1")?, IntegerOrString::Integer(1));
        assert_eq!(no_prefixed_issue(&mut "no . 123")?, IntegerOrString::Integer(123));
        assert_eq!(no_prefixed_issue(&mut "NO   456")?, IntegerOrString::Integer(456));

        Ok(())
    }
}
