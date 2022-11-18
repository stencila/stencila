use std::path::Path;

use parser_treesitter::{
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    parse_info,
    utils::{assigns_variable, uses_variable},
    ParseInfo, Parser, ParserTrait, TreesitterParser,
};

/// Tree-sitter based parser for Bash
static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_bash::language(), QUERY));

/// Tree-sitter AST query
const QUERY: &str = include_str!("query.txt");

/// A parser for Bash
pub struct BashParser {}

impl ParserTrait for BashParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Bash,
        }
    }

    fn parse(code: &str, path: Option<&Path>) -> Result<ParseInfo> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let mut dependencies = Vec::new();
        let mut dependents = Vec::new();
        'matches: for (pattern, captures) in matches.iter() {
            match pattern {
                1 => {
                    // Assigns a string variable
                    let range = Some(captures[0].range);
                    let name = captures[0].text.clone();
                    dependents.push(assigns_variable(
                        &name,
                        path,
                        Some("String".to_string()),
                        range,
                    ));
                }
                2 => {
                    // Uses a variable
                    let node = captures[0].node;
                    let range = Some(captures[0].range);
                    let name = captures[0].text.clone();

                    let mut parent = node.parent();
                    while let Some(parent_node) = parent {
                        // Skip variable_name that are the `name` of `variable_assignment`
                        if parent_node.kind() == "variable_assignment"
                            && Some(node) == parent_node.child_by_field_name("name")
                        {
                            continue 'matches;
                        }
                        parent = parent_node.parent();
                    }
                    dependencies.push(uses_variable(&name, path, None, range))
                }
                _ => (),
            }
        }

        let parse_info = parse_info(
            path,
            Self::spec().language,
            code,
            &tree,
            &["comment"],
            matches,
            0,
            dependencies,
            dependents,
        );
        Ok(parse_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    #[test]
    fn parse_bash_fragments() {
        snapshot_fixtures("fragments/bash/*.bash", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let parse_info = BashParser::parse(&code, Some(path)).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }
}
