//! Parsers to allow for terminating suffixes on reference to be ignored

use winnow::{Parser, Result, token::take_while};

/// Consume whitespace and punctuation at end of a reference item
pub fn terminator<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(0.., |c: char| c.is_ascii_punctuation() || c.is_whitespace()).parse_next(input)
}
