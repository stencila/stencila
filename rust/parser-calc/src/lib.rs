use std::path::Path;

use parser::{
    apply_comment_tags,
    common::{eyre::Result, once_cell::sync::Lazy, regex::Regex},
    formats::Format,
    hash_utils::str_seahash,
    stencila_schema::{
        ExecutionDependency, ExecutionDependencyNode, ExecutionDependencyRelation,
        ExecutionDependent, ExecutionDependentNode, ExecutionDependentRelation, Variable,
    },
    utils::remove_uses_of_assigned,
    ParseInfo, Parser, ParserTrait,
};

/// A parser for the "Calc" language
pub struct CalcParser {}

impl ParserTrait for CalcParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Calc,
        }
    }

    fn parse(code: &str, path: Option<&Path>) -> Result<ParseInfo> {
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
        let mut execution_dependencies = Vec::new();
        let mut execution_dependents = Vec::new();
        let mut comments = Vec::new();
        let mut syntax_errors = false;
        let parser = fasteval::Parser::new();
        let mut slab = fasteval::Slab::new();
        let namespace = path.map(|path| Box::new(path.to_string_lossy().to_string()));
        for (row, line) in code.lines().enumerate() {
            // Skip the line if it is blank
            if line.trim().is_empty() {
                continue;
            }

            // Parse comment line
            if line.trim_start().starts_with('#') {
                comments.push((row, line));
                continue;
            }

            // Parse line for assignments
            let (col_offset, expr) = if let Some(captures) = ASSIGN_REGEX.captures(line) {
                let name = captures.get(1).expect("Should always have group 1");
                let expr = captures.get(2).expect("Should always have group 2");
                execution_dependents.push(ExecutionDependent {
                    dependent_relation: ExecutionDependentRelation::Assigns,
                    dependent_node: ExecutionDependentNode::Variable(Variable {
                        namespace: namespace.clone(),
                        name: name.as_str().to_string(),
                        kind: Some(Box::new("Number".to_string())),
                        ..Default::default()
                    }),
                    code_location: Some([row, name.start(), row, name.end()]),
                    ..Default::default()
                });
                (expr.start(), expr.as_str())
            } else {
                (0, line)
            };

            // Parse the expression using fasteval to check there is no syntax errors in expression
            if let Err(..) = parser.parse(expr, &mut slab.ps) {
                syntax_errors = true;
                continue;
            }

            // Parse line for uses of variables
            for captures in VAR_REGEX.captures_iter(expr) {
                // Ignore function calls
                if captures.get(2).is_none() {
                    let name = captures.get(1).expect("Should always have group 1");
                    execution_dependencies.push(ExecutionDependency {
                        dependency_relation: ExecutionDependencyRelation::Uses,
                        dependency_node: ExecutionDependencyNode::Variable(Variable {
                            namespace: namespace.clone(),
                            name: name.as_str().to_string(),
                            kind: Some(Box::new("Number".to_string())),
                            ..Default::default()
                        }),
                        code_location: Some([
                            row,
                            col_offset + name.start(),
                            row,
                            col_offset + name.end(),
                        ]),
                        ..Default::default()
                    });
                }
            }

            // Add line to semantics
            semantics.push_str(line);
            semantics.push('\n');
        }

        // Remove dependencies which are also assigned within the code
        remove_uses_of_assigned(&mut execution_dependencies, &execution_dependents);

        let mut parse_info = ParseInfo {
            semantic_digest: str_seahash(&semantics)?,
            syntax_errors,
            execution_dependencies,
            execution_dependents,
            ..Default::default()
        };

        // Apply tags from comments (this needs to be done at the end because tags
        // may remove pairs if `only` is specified)
        for (row, comment) in comments {
            apply_comment_tags(&mut parse_info, comment, path, row);
        }

        Ok(parse_info)
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
            let parse_info = CalcParser::parse(&code, Some(path)).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }
}
