use lightningcss::{
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::Browsers,
    traits::ToCss,
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
    // Transform Tailwind to CSS
    let mut tailwind = TailwindBuilder::default();
    let (.., css) = tailwind.inline(tw)?;

    // Parse the CSS
    parse_css_wrapped(&css)
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
        assert_snapshot!(parse_tailwind(r"text-md text-red-200 bg-red-100")?[0]);

        // Tests of support for Tailwind classes by `tailwind-css`
        // Border radius
        assert_snapshot!(parse_tailwind(r"rounded(lg")?[0]);
        // Border width
        assert_snapshot!(parse_tailwind(r"border(2 x-4 t-8 b-0)")?[0]);
        // Border color
        assert_snapshot!(parse_tailwind(r"border(rose-400)")?[0]);
        // Border style
        assert_snapshot!(parse_tailwind(r"border(dashed)")?[0]);

        Ok(())
    }

    /// Test Tailwind variants e.g. sm, md, lg and hover
    ///
    /// Currently, handling of variants is not implemented by `tailwind-css`. These tests
    /// therefore only produce the CSS for the last variant. The snapshots are expected
    /// to break when this is implemented.
    #[test]
    fn test_parse_tailwind_variants() -> Result<()> {
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
