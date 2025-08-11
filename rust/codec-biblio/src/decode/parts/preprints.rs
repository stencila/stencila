//! Parsers to allow for some special handling of preprints

use winnow::{Parser, Result, ascii::Caseless, combinator::alt};

/// Parse the name of a preprint server
pub fn preprint_server<'s>(input: &mut &'s str) -> Result<&'s str> {
    alt((
        Caseless("arXiv").value("arXiv"),
        Caseless("bioRxiv").value("bioRxiv"),
        Caseless("medRxiv").value("medRxiv"),
    ))
    .parse_next(input)
}
