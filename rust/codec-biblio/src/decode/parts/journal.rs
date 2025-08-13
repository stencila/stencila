use codec::schema::{Inline, shortcuts::t};
use winnow::{
    Parser, Result,
    ascii::{Caseless, multispace1},
    combinator::{alt, opt, separated, terminated},
    token::take_while,
};

use crate::decode::parts::{chars::one_hyphen, preprints::preprint_server};

/// Parse a journal name not containing a comma
///
/// Allows for period within name to allow for abbreviations.
/// Parses a list of whitespace separated names and them joins them to avoid
/// trailing whitespace being consumed.
///
/// Excludes words that are all digits or punctuation to avoid consuming
/// year (e.g. "1984"), volume number (e.g. "123"), or volume & issue (e..g
/// "12(1)") etc
pub fn journal_no_comma(input: &mut &str) -> Result<Vec<Inline>> {
    alt((
        terminated(preprint_server, opt((multispace1, Caseless("preprint"))))
            .map(|server| vec![server]),
        separated(
            1..,
            alt((
                "&",
                ":",
                one_hyphen,
                take_while(1.., |c: char| c != ',' && !c.is_whitespace()).verify(|name: &str| {
                    !name
                        .chars()
                        .all(|c| c.is_numeric() || c.is_ascii_punctuation())
                }),
            )),
            multispace1,
        ),
    ))
    .map(|names: Vec<&str>| vec![t(names.join(" "))])
    .parse_next(input)
}
