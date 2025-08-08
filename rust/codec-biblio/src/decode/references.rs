//! Parsers that parse a Stencila [`Reference`] from a string

use winnow::{
    Parser, Result,
    ascii::newline,
    combinator::{alt, repeat, separated},
};

use codec::schema::Reference;

use super::apa::apa;
use super::chicago::chicago;
use super::mla::mla;

/// Parse a list of Stencila [`Reference`]s from a string
pub fn references(input: &mut &str) -> Result<Vec<Reference>> {
    separated(0.., reference, repeat::<_, _, (), _, _>(1.., newline)).parse_next(input)
}

/// Parse a Stencila [`Reference`]s from a string
pub fn reference(input: &mut &str) -> Result<Reference> {
    alt((apa, chicago, mla)).parse_next(input)
}
