use std::path::Path;

use lightningcss::{
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::Browsers,
    traits::ToCss,
};
use tailwind_css::{TailwindBuilder, TailwindErrorKind};

use common::{
    eyre::{bail, eyre, Result},
    itertools::Itertools,
};
use parser::{
    formats::Format,
    graph_triples::{execution_digest_from_content, Resource, ResourceInfo},
    utils::parse_var_interps,
    Parser, ParserTrait,
};

/// A parser for Tailwind expressions
///
/// Assumes a single line (affects location range in `Uses` relation).
/// At present does not calculate separate content and semantic strings for
/// the compile digest.
pub struct TailwindParser;

impl ParserTrait for TailwindParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Tailwind,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let relations = parse_var_interps(code, path);
        let syntax_errors = transpile_string(code).is_err().then_some(true);
        let compile_digest = execution_digest_from_content(code);

        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            None,
            None,
            syntax_errors,
            Some(compile_digest),
            None,
            None,
        );

        Ok(resource_info)
    }
}

/// Transpile a string of CSS or Tailwind to CSS
pub fn transpile_string(string: &str) -> Result<String> {
    transpile_css(string)
        .or_else(|_| transpile_css_wrapped(string))
        .or_else(|_| transpile_tailwind(string))
}

/// Transpile Tailwind directives to CSS
pub fn transpile_tailwind(tw: &str) -> Result<String> {
    // Transform Tailwind to CSS
    let mut tailwind = TailwindBuilder::default();
    let css = match tailwind.inline(tw) {
        Ok((.., css)) => css,
        Err(error) => {
            let range = error.range.as_ref().map_or_else(String::new, |range| {
                format!(" at {}-{}", range.start, range.end)
            });
            match error.kind.as_ref() {
                TailwindErrorKind::SyntaxError(msg)
                | TailwindErrorKind::TypeMismatch(msg)
                | TailwindErrorKind::RuntimeError(msg) => bail!("{}{}", msg, range),
                _ => bail!("{}", error),
            }
        }
    };

    // Transpile the CSS
    transpile_css_wrapped(&css)
}

/// Wrap CSS in a `root` selector before transpiling
fn transpile_css_wrapped(css: &str) -> Result<String> {
    transpile_css(&[":root {\n", css, "\n}"].concat())
}

/// Transpile CSS into a vector of CSS rules
fn transpile_css(css: &str) -> Result<String> {
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
        .join("\n");

    Ok(css)
}

#[cfg(test)]
mod tests {
    use parser::graph_triples::resources;
    use test_snaps::{
        insta::{assert_json_snapshot, assert_snapshot},
        snapshot_fixtures,
    };
    use test_utils::fixtures;
    use test_utils::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parse_tailwind_fragments() {
        snapshot_fixtures("fragments/tw/*.tw", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource = resources::code(path, "", "SoftwareSourceCode", Format::SQL);
            let resource_info =
                TailwindParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }

    #[test]
    fn test_transpile_css() -> Result<()> {
        // Nesting
        assert_snapshot!(transpile_css(
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
        )?);

        // Media queries
        assert_snapshot!(transpile_css(
            r#"
            @media (min-width: 640px) {
                :root {
                    font-size: 1rem;
                    line-height: 1.5rem;
                }
            }
            "#,
        )?);

        Ok(())
    }

    #[test]
    fn test_transpile_tailwind() -> Result<()> {
        // Basic
        assert_snapshot!(transpile_tailwind(r"text-md text-red-200 bg-red-100")?);

        // Tests of support for Tailwind classes by `tailwind-css`
        // Border radius
        assert_snapshot!(transpile_tailwind(r"rounded(lg")?);
        // Border width
        assert_snapshot!(transpile_tailwind(r"border(2 x-4 t-8 b-0)")?);
        // Border color
        assert_snapshot!(transpile_tailwind(r"border(rose-400)")?);
        // Border style
        assert_snapshot!(transpile_tailwind(r"border(dashed)")?);

        Ok(())
    }

    /// Test Tailwind variants e.g. sm, md, lg and hover
    ///
    /// Currently, handling of variants is not implemented by `tailwind-css`. These tests
    /// therefore only produce the CSS for the last variant. The snapshots are expected
    /// to break when this is implemented.
    #[test]
    fn test_transpile_tailwind_variants() -> Result<()> {
        // Four ways to do same thing with breakpoints
        assert_snapshot!(transpile_tailwind(
            r"sm:text-md sm:text(md) sm:(text(md)) sm:(text-md)"
        )?);

        // As above but using different sizes
        assert_snapshot!(transpile_tailwind(
            r"sm:text-md md:text(lg) lg:(text(xl)) xl:(text-5xl)"
        )?);

        Ok(())
    }

    #[test]
    fn test_transpile_string() -> Result<()> {
        let css1 = transpile_string(r"font-size: 1rem; line-height: 1.5rem")?;
        let css2 = transpile_string(r"text-md")?;
        assert_eq!(css1, css2);

        Ok(())
    }
}
