use once_cell::sync::Lazy;
use parser_treesitter::{
    eyre::Result,
    formats::Format,
    graph_triples::{relations, resources, Resource, ResourceInfo},
    resource_info, Parser, ParserTrait, TreesitterParser,
};
use std::path::Path;

/// Tree-sitter based parser for Rust
static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_rust::language(), QUERY));

/// Tree-sitter AST query
const QUERY: &str = include_str!("query.txt");

/// A parser for Rust
pub struct RustParser {}

impl ParserTrait for RustParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Rust.spec().title,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let relations = matches
            .iter()
            .filter_map(|(pattern, captures)| match pattern {
                0 | 1 | 2 => {
                    // Assigns a variable
                    let range = captures[0].range;
                    let name = captures[0].text.clone();
                    Some((
                        relations::assigns(range),
                        resources::symbol(path, &name, ""),
                    ))
                }
                3 => {
                    // Uses a variable
                    let node = captures[0].node;
                    let range = captures[0].range;
                    let symbol = captures[0].text.clone();

                    if let Some(parent_node) = node.parent() {
                        let kind = parent_node.kind();
                        if
                        // Skip names of `static` and `const`s
                        ((kind == "const_item" || kind == "static_item")
                               && Some(node) == parent_node.child_by_field_name("name"))
                        // Skip names of `let`s
                        || (kind == "let_declaration"
                            && Some(node) == parent_node.child_by_field_name("pattern"))
                        // Skip any part of a scoped identifier e.g. Vec::new
                        || kind == "scoped_identifier"
                        // Skip names of macros
                        || kind == "macro_invocation"
                            && Some(node) == parent_node.child_by_field_name("macro")
                        {
                            return None;
                        }
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
            &tree,
            &["comment"],
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
    fn parse_rust_fragments() {
        snapshot_fixtures("fragments/rust/*.rs", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource =
                resources::code(path, "", "SoftwareSourceCode", Some("Rust".to_string()));
            let resource_info = RustParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }
}
