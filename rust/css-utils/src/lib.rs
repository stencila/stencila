use lightningcss::{
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::Browsers,
    traits::ToCss,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, take_while1},
    character::{
        complete::{multispace0, multispace1}, is_alphanumeric,
        streaming::char,
    },
    combinator::{all_consuming, map, peek, recognize},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, terminated, tuple},
    IResult,
};
use tailwind_css::TailwindBuilder;

use common::{
    eyre::{eyre, Result},
    itertools::Itertools,
};

/// Parse a string of CSS or Tailwind
pub fn parse_string(string: &str) -> Result<Vec<String>> {
    parse_css(string)
        .or_else(|_| parse_css_wrapped(string))
        .or_else(|_| parse_tailwind(string))
}

/// Parse Tailwind directives
fn parse_tailwind(tw: &str) -> Result<Vec<String>> {
    // Expand any Tailwind groups e.g. `text(lg red-100)` into `text-lg text-red-100`
    let result = expand_tailwind_groups(tw.trim());
    let directives = result
        .map_err(|error| eyre!("While parsing Tailwind groups: {}", error.to_string()))?
        .1;

    // Transform each Tailwind directive into CSS, wrapping the result in
    // `@media` queries if necessary
    let mut tailwind = TailwindBuilder::default();
    let mut all = String::new();
    for directive in directives {
        let (.., css) = tailwind.inline(&directive).unwrap();
        let css = if let Some((breakpoint, ..)) =
            directive.splitn(2, ':').collect_tuple::<(&str, &str)>()
        {
            let px = match breakpoint {
                "sm" => 640,
                "md" => 768,
                "lg" => 1024,
                "xl" => 1280,
                "2xl" => 1536,
                _ => 768,
            };
            [
                "@media (min-width: ",
                &px.to_string(),
                "px) {\n",
                &css,
                "\n}\n",
            ]
            .concat()
        } else {
            css
        };
        all.push_str(&css);
    }

    // Parse the CSS
    parse_css_wrapped(&all)
}

/// Expand any Tailwind directive
fn expand_tailwind_groups(input: &str) -> IResult<&str, Vec<String>> {
    all_consuming(separated_list0(
        multispace1,
        alt((
            tailwind_directive_group,
            tailwind_variant_group,
            tailwind_directive,
        )),
    ))(input)
}

/// Recognize a Tailwind directive
///
/// Should parse all directives e.g.
///   border-2 border-red-100 sm:border-opacity-50 sm:hover:border-dashed
fn tailwind_directive(input: &str) -> IResult<&str, String> {
    map(
        recognize(take_while1(|c: char| {
            is_alphanumeric(c as u8) || c == '-' || c == ':'
        })),
        |dir: &str| -> String { dir.to_string() },
    )(input)
}

/// Expand a Tailwind directive group
///
/// Implements the directive grouping of https://twind.dev e.g.
///    border(2 red-100)
fn tailwind_directive_group(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            recognize(terminated(
                take_while1(|c: char| is_alphanumeric(c as u8) || c == '-' || c == ':'),
                peek(is_not(":")),
            )),
            delimited(
                tuple((char('('), multispace0)),
                separated_list1(
                    multispace1,
                    alt((
                        tailwind_directive_group,
                        tailwind_variant_group,
                        tailwind_directive,
                    )),
                ),
                tuple((multispace0, char(')'))),
            ),
        )),
        |(prefix, items)| -> String {
            items
                .iter()
                .map(|item| {
                    if prefix.ends_with('-') || item.starts_with('-') {
                        [prefix, item].concat()
                    } else {
                        [prefix, "-", item].concat()
                    }
                })
                .join(" ")
        },
    )(input)
}

/// Expand a Tailwind variant group
///
/// Implements the variant grouping whcih allows for nested directive
/// groups but not nested variant groups e.g.
///    sm:(border(2 red-100))
fn tailwind_variant_group(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            recognize(tuple((
                take_while1(|c: char| is_alphanumeric(c as u8)),
                char(':'),
            ))),
            delimited(
                tuple((char('('), multispace0)),
                separated_list1(
                    multispace1,
                    alt((tailwind_directive_group, tailwind_directive)),
                ),
                tuple((multispace0, char(')'))),
            ),
        )),
        |(prefix, items)| -> String { items.iter().map(|item| [prefix, item].concat()).join(" ") },
    )(input)
}

/// Wrap CSS in a `root` selector before parsing
fn parse_css_wrapped(css: &str) -> Result<Vec<String>> {
    parse_css(&[":root {\n", css, "\n}"].concat())
}

/// Parse and transform CSS into a vector of CSS rules
fn parse_css(css: &str) -> Result<Vec<String>> {
    let targets = Some(Browsers {
        ..Default::default()
    });

    // Parse the CSS into rules
    let mut sheet = StyleSheet::parse(
        css,
        ParserOptions {
            nesting: true,
            ..Default::default()
        },
    )
    .map_err(|error| eyre!("Error parsing CSS: {}\n{}", error.to_string(), css))?;

    // Optimize the rules
    sheet.minify(MinifyOptions {
        ..Default::default()
    })?;

    // Generate a vector of CSS rules
    let css = sheet
        .rules
        .0
        .iter()
        .filter_map(|rule| {
            rule.to_css_string(PrinterOptions {
                // It is important to specify targets here, even just defaults, so that nested rules
                // get expanded
                targets,
                // Do not minify during development so debugging easier and snapshots easier to read
                minify: !cfg!(debug_assertions),
                ..Default::default()
            })
            .ok()
        })
        .collect_vec();

    Ok(css)
}

#[cfg(test)]
mod tests {
    use test_snaps::insta::assert_snapshot;
    use test_utils::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_css() -> Result<()> {
        // Nesting
        assert_snapshot!(parse_css(
            r#"
            .foo {
                color: pink;
                & .bar {
                    color: red;
                    & .baz .quax {
                        color: orange;
                    }
                }
            }
            "#,
        )?
        .join("\n\n"));

        // Media queries
        assert_snapshot!(parse_css(
            r#"
            @media (min-width: 640px) {
                :root {
                    font-size: 1rem;
                    line-height: 1.5rem;
                }
            }
            "#,
        )?
        .join("\n\n"));

        Ok(())
    }

    #[test]
    fn test_parse_tailwind() -> Result<()> {
        // Basic
        assert_snapshot!(parse_tailwind(r"text-md text-red-200 bg-red-100")?.join("\n\n"));

        // Four ways to do same thing with breakpoints
        assert_snapshot!(
            parse_tailwind(r"sm:text-md sm:text(md) sm:(text(md)) sm:(text-md)")?.join("\n\n")
        );

        // As above but using different sizes
        assert_snapshot!(
            parse_tailwind(r"sm:text-md md:text(lg) lg:(text(xl)) xl:(text-5xl)")?.join("\n\n")
        );

        Ok(())
    }

    #[test]
    fn test_parse_string() -> Result<()> {
        let css1 = parse_string(r"font-size: 1rem; line-height: 1.5rem")?;
        let css2 = parse_string(r"text-md")?;
        assert_eq!(css1, css2);

        Ok(())
    }
}
