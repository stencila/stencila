use winnow::{
    Parser, Result,
    ascii::{multispace0, multispace1},
    combinator::alt,
};

/// Parse a separator between parts of a reference
///
/// This is a lenient parser for anything that may be used as a separator
/// between parts of a reference. Making it lenient allows parsers
/// to be more robust to deviations in punctuation and whitespace.
pub fn separator<'s>(input: &mut &'s str) -> Result<&'s str> {
    alt((
        (multispace0, alt((",", ".", ";", ":")), multispace0).take(),
        multispace1,
    ))
    .parse_next(input)
}
