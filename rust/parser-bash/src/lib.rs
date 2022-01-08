use once_cell::sync::Lazy;
use parser_treesitter::{
    eyre::Result,
    formats::Format,
    graph_triples::{relations, resources, Resource, ResourceInfo},
    resource_info, Parser, ParserTrait, TreesitterParser,
};
use std::path::Path;

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
            language: Format::Bash.spec().title,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let relations = matches
            .iter()
            .filter_map(|(pattern, captures)| match pattern {
                0 => {
                    // Assigns a string variable
                    let range = captures[0].range;
                    let name = captures[0].text.clone();
                    Some((
                        relations::assigns(range),
                        resources::symbol(path, &name, "String"),
                    ))
                }
                1 => {
                    // Uses a variable
                    let node = captures[0].node;
                    let range = captures[0].range;
                    let symbol = captures[0].text.clone();

                    let mut parent = node.parent();
                    while let Some(parent_node) = parent {
                        // Skip variable_name that are the `name` of `variable_assignment`
                        if parent_node.kind() == "variable_assignment"
                            && Some(node) == parent_node.child_by_field_name("name")
                        {
                            return None;
                        }
                        parent = parent_node.parent();
                    }

                    Some((relations::uses(range), resources::symbol(path, &symbol, "")))
                }
                _ => None,
            })
            .collect();

        let resource_info = resource_info(
            resource,
            path,
            &Self::spec().language,
            code,
            matches,
            0,
            relations,
        );
        Ok(resource_info)
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
            let resource =
                resources::code(path, "", "SoftwareSourceCode", Some("Bash".to_string()));
            let resource_info = BashParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }
}
