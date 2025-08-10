use winnow::{
    Parser, Result,
    ascii::{Caseless, multispace0},
    combinator::{alt, not, opt, preceded},
    token::take_while,
};

/// Parse a URL
pub fn url(input: &mut &str) -> Result<String> {
    preceded(
        opt((Caseless("URL"), multispace0, opt(":"), multispace0)),
        (
            alt(("https://", "http://")),
            preceded(
                not(alt(("doi.org/", "dx.doi.org/", "www.doi.org/"))),
                take_while(1.., |c: char| !c.is_ascii_whitespace()),
            ),
        ),
    )
    .map(|(prefix, suffix): (&str, &str)| {
        [prefix, suffix.trim_end_matches(['.', ',', ';'])].concat()
    })
    .parse_next(input)
}
