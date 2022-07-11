use std::path::Path;

use parser_treesitter::{
    captures_as_args_map,
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    graph_triples::{relations, resources, Resource, ResourceInfo},
    path_utils, resource_info,
    utils::{is_quoted, remove_quotes},
    Parser, ParserTrait, TreesitterParser,
};

/// Tree-sitter based parser for Python
static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_python::language(), QUERY));

/// Tree-sitter AST query
const QUERY: &str = include_str!("query.txt");

mod ignores;
use ignores::USE_IGNORE;

/// A parser for Python
pub struct PyParser {}

impl ParserTrait for PyParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Python.spec().title,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let relations = matches
            .iter()
            .filter_map(|(pattern, captures)| match pattern {
                1 | 2 => {
                    // Imports a module
                    let range = captures[0].range;
                    let module = &captures[0].text;
                    let path = path_utils::merge(path, [module, ".py"].concat());
                    let object = match path.exists() {
                        true => resources::file(&path),
                        false => resources::module("python", module),
                    };
                    Some((relations::imports(range), object))
                }
                3 => {
                    // Opens a file for reading or writing
                    let args = captures_as_args_map(captures);
                    if let Some(file) = args.get("0").or_else(|| args.get("file")) {
                        if !is_quoted(&file.text) {
                            return None;
                        }
                        let path = path_utils::merge(path, remove_quotes(&file.text));
                        let range = file.range;
                        if let Some(mode) = args.get("1").or_else(|| args.get("mode")) {
                            if !is_quoted(&mode.text) {
                                return None;
                            }
                            let mode = remove_quotes(&mode.text);
                            if mode.starts_with('w') || mode.starts_with('a') {
                                Some((relations::writes(range), resources::file(&path)))
                            } else {
                                Some((relations::reads(range), resources::file(&path)))
                            }
                        } else {
                            Some((relations::reads(range), resources::file(&path)))
                        }
                    } else {
                        None
                    }
                }
                4 | 5 => {
                    // Assigns a symbol at the top level of the module
                    let range = captures[0].range;
                    let name = captures[0].text.clone();
                    let kind = match pattern {
                        4 => match captures[1].node.kind() {
                            "true" | "false" => "Boolean",
                            "integer" => "Integer",
                            "float" => "Number",
                            "string" => "String",
                            "list" => "Array",
                            "dictionary" => "Object",
                            "lambda" => "Function",
                            _ => "",
                        },
                        5 => "Function",
                        _ => unreachable!(),
                    };

                    let symbol = resources::symbol(path, &name, kind);
                    let relation = match pattern {
                        4 => relations::assigns(range),
                        5 => relations::declares(range),
                        _ => unreachable!(),
                    };
                    Some((relation, symbol))
                }
                6 => {
                    // Uses an identifier assigned elsewhere
                    let node = captures[0].node;
                    let range = captures[0].range;
                    let symbol = captures[0].text.clone();

                    if USE_IGNORE.contains(&symbol.as_str()) {
                        return None;
                    }

                    let mut parent = node.parent();
                    while let Some(parent_node) = parent {
                        match parent_node.kind() {
                            // Skip identifiers that are the `left` of an assignment
                            "assignment" => {
                                if Some(node) == parent_node.child_by_field_name("left") {
                                    return None;
                                }
                            }
                            // Skip any identifier used in a function parameter
                            "parameters" | "lambda_parameters" => {
                                return None;
                            }
                            // Skip identifiers that are the `name` of a keyword argument
                            "keyword_argument" => {
                                if Some(node) == parent_node.child_by_field_name("name") {
                                    return None;
                                }
                            }
                            // Skip identifiers that are an `attribute`
                            "object" | "function" | "attribute" => {
                                if Some(node) == parent_node.child_by_field_name("attribute") {
                                    return None;
                                }
                            }
                            // Skip identifiers that are the `left` of a for loop, or that refer to it
                            // within the loop
                            "for_statement" => {
                                if let Some(left) = parent_node.child_by_field_name("left") {
                                    if left == node || left.utf8_text(code).unwrap() == symbol {
                                        return None;
                                    }
                                }
                            }
                            // Skip identifiers within these...
                            "import_statement"
                            | "import_from_statement"
                            | "function_definition"
                            | "lambda" => return None,
                            // Skip identifiers that are the identifier in an `as_pattern_target`
                            "as_pattern_target" => {
                                return None;
                            }
                            // Skip any references to the `as_pattern_target` within `with` statements.
                            // This requires use to walk up the ancestors looking for a `with_statement`
                            // and then checking if the alias is the same as the identifier.
                            _ => {
                                let mut ancestor = parent_node;
                                loop {
                                    if ancestor.kind() == "with_statement" {
                                        if let Some(alias) = ancestor
                                            .child(1) // "with_clause"
                                            .and_then(|node| node.child(0)) // "with_item"
                                            .and_then(|node| node.child_by_field_name("value")) // "as_pattern"
                                            .and_then(|node| node.child_by_field_name("alias")) // "as_pattern_target"
                                            .and_then(|node| node.utf8_text(code).ok())
                                        {
                                            if symbol == alias {
                                                return None;
                                            }
                                        }
                                    }
                                    if let Some(parent) = ancestor.parent() {
                                        ancestor = parent;
                                    } else {
                                        break;
                                    }
                                }
                            }
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
    fn parse_py_fragments() {
        snapshot_fixtures("fragments/py/*.py", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource =
                resources::code(path, "", "SoftwareSourceCode", Some("Python".to_string()));
            let resource_info = PyParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }
}
