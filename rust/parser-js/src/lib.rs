use std::path::Path;

use parser_treesitter::{
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    graph_triples::{relations, resources, Pair, Resource, ResourceInfo},
    path_utils, resource_info,
    utils::remove_quotes,
    Capture, Parser, ParserTrait, TreesitterParser,
};

/// Tree-sitter based parser for JavaScript
static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_javascript::language(), QUERY));

/// Tree-sitter AST query for JavaScript
///
/// Made public for use by `parser-ts`.
pub const QUERY: &str = include_str!("query.scm");

/// A parser for JavaScript
pub struct JsParser {}

impl ParserTrait for JsParser {
    fn spec() -> Parser {
        Parser {
            language: Format::JavaScript.spec().title,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let relations = matches
            .iter()
            .filter_map(|(pattern, capture)| handle_patterns(path, code, pattern, capture))
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

/// Handle a pattern match
///
/// Made public for use by `parser-ts`.
pub fn handle_patterns(
    path: &Path,
    code: &[u8],
    pattern: &usize,
    captures: &[Capture],
) -> Option<Pair> {
    match pattern {
        1 | 2 => {
            // Imports a module using `import` or `require`
            let capture = match pattern {
                1 => &captures[0],
                2 => &captures[1],
                _ => unreachable!(),
            };
            let range = capture.range;
            let module = remove_quotes(&capture.text.clone());
            let object = if module.starts_with("./") {
                resources::file(&path_utils::merge(path, &[&module, ".js"].concat()))
            } else {
                resources::module("javascript", &module)
            };
            Some((relations::imports(range), object))
        }
        3 => {
            // Reads a file
            Some((
                relations::reads(captures[1].range),
                resources::file(&path_utils::merge(path, remove_quotes(&captures[1].text))),
            ))
        }
        4 => {
            // Writes a file
            Some((
                relations::writes(captures[1].range),
                resources::file(&path_utils::merge(path, remove_quotes(&captures[1].text))),
            ))
        }
        5 | 6 => {
            // Declares a symbol at the top level of the module
            let range = captures[0].range;
            let name = captures[0].text.clone();
            let kind = match pattern {
                5 => node_kind(captures[1].node.kind()),
                6 => "Function",
                _ => unreachable!(),
            };
            Some((
                relations::declares(range),
                resources::symbol(path, &name, kind),
            ))
        }
        7 => {
            // Assigns a symbol at the top level of the module
            let range = captures[0].range;
            let name = captures[0].text.clone();
            let kind = node_kind(captures[1].node.kind());
            Some((
                relations::assigns(range),
                resources::symbol(path, &name, kind),
            ))
        }
        8 => {
            // Uses an identifier assigned elsewhere
            let node = captures[0].node;
            let range = captures[0].range;
            let symbol = captures[0].text.clone();

            let mut parent = node.parent();
            while let Some(parent_node) = parent {
                match parent_node.kind() {
                    // Skip identifiers in import statements
                    // Could just skip children of `import_statement`, but specifying others in tree
                    // results in an earlier return while walking up tree.
                    "import_statement" | "import_clause" | "named_imports" | "import_specifier" => {
                        return None
                    }
                    // Skip identifiers that are the `name` of a declaration
                    "variable_declarator" => {
                        if Some(node) == parent_node.child_by_field_name("name") {
                            return None;
                        }
                    }
                    // Skip identifiers that are the `left` of an assignment
                    "assignment_expression" => {
                        if Some(node) == parent_node.child_by_field_name("left") {
                            return None;
                        }
                    }
                    // Skip any identifier used in a function
                    "function_declaration" | "arrow_function" | "formal_parameters" => {
                        return None;
                    }
                    // Skip identifiers that are the `left` of a for in loop, or that refer to it
                    // within the loop
                    "for_in_statement" => {
                        if let Some(left) = parent_node.child_by_field_name("left") {
                            if left == node || left.utf8_text(code).unwrap() == symbol {
                                return None;
                            }
                        }
                    }
                    // Skip identifiers that are the `name` of the variable in a for loop, or
                    // that refer to it within the loop
                    "for_statement" => {
                        if let Some(name) = parent_node
                            .child_by_field_name("initializer")
                            .and_then(|node| node.named_child(0)) // variable_declarator
                            .and_then(|node| node.child_by_field_name("name"))
                        {
                            if name == node || name.utf8_text(code).unwrap() == symbol {
                                return None;
                            }
                        }
                    }
                    _ => {}
                }
                parent = parent_node.parent();
            }

            Some((relations::uses(range), resources::symbol(path, &symbol, "")))
        }
        _ => None,
    }
}

// Translate a `tree-sitter-javascript` AST node `kind` into a Stencila node `type`
fn node_kind(kind: &str) -> &str {
    match kind {
        "true" | "false" => "Boolean",
        "number" => "Number",
        "string" => "String",
        "array" => "Array",
        "object" => "Object",
        "arrow_function" => "Function",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    #[test]
    fn parse_js_fragments() {
        snapshot_fixtures("fragments/js/*.js", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource = resources::code(
                path,
                "",
                "SoftwareSourceCode",
                Some("JavaScript".to_string()),
            );
            let resource_info = JsParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }
}
