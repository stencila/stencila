use winnow::{
    Parser, Result,
    combinator::{alt, not, preceded},
    token::take_while,
};

/// Parse a URL
pub fn url(input: &mut &str) -> Result<String> {
    (
        alt(("https://", "http://")),
        preceded(
            not(alt(("doi.org/", "dx.doi.org/", "www.doi.org/"))),
            take_while(1.., |c: char| !c.is_ascii_whitespace()),
        ),
    )
        .map(|(prefix, suffix): (&str, &str)| {
            [prefix, suffix.trim_end_matches(['.', ',', ';'])].concat()
        })
        .parse_next(input)
}
