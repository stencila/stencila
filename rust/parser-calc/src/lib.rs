use std::path::Path;

use parser::{
    common::{eyre::Result, once_cell::sync::Lazy, regex::Regex},
    formats::Format,
    graph_triples::{
        relations,
        resources::{self, ResourceDigest},
        Resource, ResourceInfo,
    },
    utils::apply_tags,
    Parser, ParserTrait,
};

/// A parser for the "Calc" language
pub struct CalcParser {}

impl ParserTrait for CalcParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Calc,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        static ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\s*([a-zA-Z_][a-zA-Z_0-9]*)\s*=(.*)").expect("Unable to create regex")
        });

        // Although we could parse the expression part of each line using `fasteval` and looking
        // for `EVar` nodes in the parse tree, it seems that walking the `fasteval`
        // parse tree is not trivial. So, this uses regex to get variable names (avoiding function
        // names)
        static VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(\b[a-zA-Z_][a-zA-Z_0-9]*\b)(\s*\()?").expect("Unable to create regex")
        });

        // The semantic content of the code (includes the language name and ignores comments)
        let mut semantics = Self::spec().language.to_string();
        let mut comments = Vec::new();
        let mut syntax_errors = None;
        let parser = fasteval::Parser::new();
        let mut slab = fasteval::Slab::new();
        let relations = code
            .split('\n')
            .enumerate()
            .fold(Vec::new(), |mut pairs, (row, line)| {
                // Skip the line if it is blank
                if line.trim().is_empty() {
                    return pairs;
                }

                // Parse comment line
                if line.trim_start().starts_with('#') {
                    comments.push((row, line));
                    return pairs;
                }

                // Parse line for assignments
                let (start, expr) = if let Some(captures) = ASSIGN_REGEX.captures(line) {
                    let symbol = captures.get(1).expect("Should always have group 1");
                    let expr = captures.get(2).expect("Should always have group 2");
                    pairs.push((
                        relations::assigns((row, symbol.start(), row, symbol.end())),
                        resources::symbol(path, symbol.as_str(), "Number"),
                    ));
                    (expr.start(), expr.as_str())
                } else {
                    (0, line)
                };

                // Parse the expression using fasteval to check there is no syntax errors in expression
                if let Err(..) = parser.parse(expr, &mut slab.ps) {
                    syntax_errors = Some(true);
                    return pairs;
                }

                // Parse line for uses of variables
                for captures in VAR_REGEX.captures_iter(expr) {
                    if captures.get(2).is_none() {
                        let symbol = captures.get(1).expect("Should always have group 1");
                        pairs.push((
                            relations::uses((
                                row,
                                start + symbol.start(),
                                row,
                                start + symbol.end(),
                            )),
                            resources::symbol(path, symbol.as_str(), "Number"),
                        ))
                    }
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
            syntax_errors,
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
                Some("Number".to_string()),
                &mut resource_info,
            );
        }

        Ok(resource_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    #[test]
    fn calc_fragments() {
        snapshot_fixtures("fragments/calc/*.calc", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource = resources::code(path, "", "SoftwareSourceCode", Format::Calc);
            let resource_info = CalcParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }
}
