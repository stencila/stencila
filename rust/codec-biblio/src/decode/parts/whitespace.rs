use winnow::{Parser, Result, token::take_while};

/// Parse zero or more whitespace
///
/// Differs from multispace0 in that is also allows for no-break spaces (U+00A0)
pub fn whitespace0<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(0.., |c: char| c.is_whitespace()).parse_next(input)
}

/// Parse one or more whitespace
///
/// Differs from multispace1 in that is also allows for no-break spaces (U+00A0)
pub fn whitespace1<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1.., |c: char| c.is_whitespace()).parse_next(input)
}
