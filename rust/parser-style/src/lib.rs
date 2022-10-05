use std::path::Path;

use lightningcss::{
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::Browsers,
    traits::ToCss,
};
use tailwind_css::TailwindBuilder;

use common::{
    eyre::{eyre, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    regex::Regex,
};
use parser::{
    formats::Format,
    graph_triples::{
        relations,
        resources::{self, ResourceDigest},
        Resource, ResourceInfo,
    },
    utils::apply_tags,
    Parser, ParserTrait,
};

/// Regex for detecting document variables used in styles
pub static VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\$([a-zA-Z_][a-zA-Z_0-9]*)\b)|(?:\$\{\s*([a-zA-Z_][a-zA-Z_0-9]*)\s*\})")
        .expect("Unable to create regex")
});

/// A parser for the "Style" language
pub struct StyleParser;

impl ParserTrait for StyleParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Style,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        // The semantic content of the code (includes the language name and ignores comments)
        let mut semantics = Self::spec().language.to_string();
        let mut comments = Vec::new();
        let relations = code
            .split('\n')
            .enumerate()
            .fold(Vec::new(), |mut pairs, (row, line)| {
                // Skip the line if it is blank
                if line.trim().is_empty() {
                    return pairs;
                }

                // Parse comment line
                if line.trim_start().starts_with("//") {
                    comments.push((row, line));
                    return pairs;
                }

                let (start, expr) = (0, line);

                // Parse line for uses of variables
                for captures in VAR_REGEX.captures_iter(expr) {
                    let symbol = captures
                        .get(1)
                        .or_else(|| captures.get(2))
                        .expect("Should always have one group");
                    pairs.push((
                        relations::uses((row, start + symbol.start(), row, start + symbol.end())),
                        resources::symbol(path, symbol.as_str(), ""),
                    ))
                }

                // Add line to semantics
                semantics.push_str(line);
                semantics.push('\n');

                pairs
            });

        let mut resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            None,
            None,
            Some(ResourceDigest::from_strings(code, Some(&semantics))),
            None,
            None,
        );

        // Apply tags from comments (this needs to be done at the end because tags
        // may remove pairs if `only` is specified)
        for (row, line) in comments {
            apply_tags(
                path,
                Self::spec().language,
                row,
                line,
                None,
                &mut resource_info,
            );
        }

        Ok(resource_info)
    }
}

/// Transpile a string of CSS or Tailwind to CSS
pub fn transpile_string(string: &str) -> Result<Vec<String>> {
    transpile_css(string)
        .or_else(|_| transpile_css_wrapped(string))
        .or_else(|_| transpile_tailwind(string))
}

/// Transpile Tailwind directives to CSS
fn transpile_tailwind(tw: &str) -> Result<Vec<String>> {
    // Transform Tailwind to CSS
    let mut tailwind = TailwindBuilder::default();
    let (.., css) = tailwind.inline(tw)?;

    // Parse the CSS
    transpile_css_wrapped(&css)
}

/// Wrap CSS in a `root` selector before transpiling
fn transpile_css_wrapped(css: &str) -> Result<Vec<String>> {
    transpile_css(&[":root {\n", css, "\n}"].concat())
}

/// Transpile CSS into a vector of CSS rules
fn transpile_css(css: &str) -> Result<Vec<String>> {
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
    use test_snaps::{
        insta::{assert_json_snapshot, assert_snapshot},
        snapshot_fixtures,
    };
    use test_utils::fixtures;
    use test_utils::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parse_style_fragments() {
        snapshot_fixtures("fragments/style/*.style", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource = resources::code(path, "", "SoftwareSourceCode", Format::SQL);
            let resource_info = StyleParser::parse(resource, path, &code).expect("Unable to parse");
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
        )?
        .join("\n\n"));

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
        )?
        .join("\n\n"));

        Ok(())
    }

    #[test]
    fn test_transpile_tailwind() -> Result<()> {
        // Basic
        assert_snapshot!(transpile_tailwind(r"text-md text-red-200 bg-red-100")?[0]);

        // Tests of support for Tailwind classes by `tailwind-css`
        // Border radius
        assert_snapshot!(transpile_tailwind(r"rounded(lg")?[0]);
        // Border width
        assert_snapshot!(transpile_tailwind(r"border(2 x-4 t-8 b-0)")?[0]);
        // Border color
        assert_snapshot!(transpile_tailwind(r"border(rose-400)")?[0]);
        // Border style
        assert_snapshot!(transpile_tailwind(r"border(dashed)")?[0]);

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
        assert_snapshot!(
            transpile_tailwind(r"sm:text-md sm:text(md) sm:(text(md)) sm:(text-md)")?.join("\n\n")
        );

        // As above but using different sizes
        assert_snapshot!(transpile_tailwind(
            r"sm:text-md md:text(lg) lg:(text(xl)) xl:(text-5xl)"
        )?
        .join("\n\n"));

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
