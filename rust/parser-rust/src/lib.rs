use std::path::Path;

use parser_treesitter::{
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    parse_info,
    utils::{declares_variable, uses_variable},
    ParseInfo, Parser, ParserTrait, TreesitterParser,
};

/// Tree-sitter based parser for Rust
static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_rust::language(), QUERY));

/// Tree-sitter AST query
const QUERY: &str = include_str!("query.scm");

/// A parser for Rust
pub struct RustParser {}

impl ParserTrait for RustParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Rust,
        }
    }

    fn parse(code: &str, path: Option<&Path>) -> Result<ParseInfo> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let mut dependencies = Vec::new();
        let mut dependents = Vec::new();
        for (pattern, captures) in matches.iter() {
            match pattern {
                1 | 2 | 3 => {
                    // Declares a variable
                    let range = captures[0].range;
                    let name = captures[0].text.clone();
                    dependents.push(declares_variable(&name, path, None, Some(range)))
                }
                4 => {
                    // Uses a variable
                    let node = captures[0].node;
                    let range = captures[0].range;
                    let name = captures[0].text.clone();

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
                            continue;
                        }
                    }
                    dependencies.push(uses_variable(&name, path, None, Some(range)))
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
    fn parse_rust_fragments() {
        snapshot_fixtures("fragments/rust/*.rs", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let parse_info = RustParser::parse(&code, Some(path)).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }
}
