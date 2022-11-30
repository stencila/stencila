use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use parser_treesitter::{
    captures_as_args_map, child_text,
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    parse_info, path_utils,
    utils::{
        assigns_variable, imports_module, is_quoted, reads_file, remove_quotes, uses_variable,
        writes_file,
    },
    ParseInfo, Parser, ParserTrait, TreesitterParser,
};

/// Tree-sitter based parser for R
static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_r::language(), QUERY));

/// Tree-sitter AST query
const QUERY: &str = include_str!("query.scm");

mod ignores;
use ignores::USE_IGNORE;

/// A parser for Python
pub struct RParser {}

impl ParserTrait for RParser {
    fn spec() -> Parser {
        Parser {
            language: Format::R,
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
                    // Imports a package using `library` or `require`
                    let args = captures_as_args_map(captures);
                    if let Some(package) = args.get("0").or_else(|| args.get("package")) {
                        if let Some(is_char) = args.get("character.only") {
                            if is_char.text.starts_with('T') && !is_quoted(&package.text) {
                                continue 'matches;
                            }
                        } else if is_quoted(&package.text) {
                            continue 'matches;
                        }
                        dependencies.push(imports_module(
                            &remove_quotes(&package.text),
                            Some(package.range),
                        ))
                    }
                }
                2 => {
                    // Reads a file
                    let args = captures_as_args_map(captures);
                    if let Some(file) = args.get("0").or_else(|| args.get("file")) {
                        let file_path = path_utils::merge(
                            path.map(PathBuf::from).unwrap_or_else(|| {
                                current_dir().expect("Should be able to get pwd")
                            }),
                            remove_quotes(&file.text),
                        );
                        dependencies.push(reads_file(&file_path, Some(file.range)))
                    }
                }
                3 => {
                    // Writes a file
                    let args = captures_as_args_map(captures);
                    if let Some(file) = args.get("1").or_else(|| args.get("file")) {
                        let file_path = path_utils::merge(
                            path.map(PathBuf::from).unwrap_or_else(|| {
                                current_dir().expect("Should be able to get pwd")
                            }),
                            remove_quotes(&file.text),
                        );
                        dependents.push(writes_file(&file_path, Some(file.range)))
                    }
                }
                4 | 5 => {
                    // Assigns a symbol at the top level of the module
                    let range = captures[0].range;
                    let name = captures[0].text.clone();
                    let value = captures[1].node;
                    let kind = match value.kind() {
                        "true" | "false" => Some("Boolean".to_string()),
                        "integer" => Some("Integer".to_string()),
                        "float" => Some("Number".to_string()),
                        "string" => Some("String".to_string()),
                        "function_definition" => Some("Function".to_string()),
                        "call" => match child_text(value, "function", code) {
                            "data.frame" | "read.csv" | "read.csv2" | "read.delim"
                            | "read.table" => Some("Datatable".to_string()),
                            _ => None,
                        },
                        _ => None,
                    };
                    dependents.push(assigns_variable(&name, path, kind, Some(range)))
                }
                6 => {
                    // Uses a function or variable
                    let node = captures[0].node;
                    let range = captures[0].range;
                    let symbol = captures[0].text.clone();

                    let mut parent = node.parent();
                    while let Some(parent_node) = parent {
                        match parent_node.kind() {
                            // Skip identifiers that are the `name` of an assignment
                            "left_assignment" | "equals_assignment" | "super_assignment" => {
                                if let Some(name) = parent_node.child_by_field_name("name") {
                                    if name == node {
                                        continue 'matches;
                                    }
                                }
                            }
                            // Skip identifiers that are the `name` of a function call argument
                            "arguments" => {
                                let mut cursor = node.walk();
                                for name in parent_node.children_by_field_name("name", &mut cursor)
                                {
                                    if name == node {
                                        continue 'matches;
                                    }
                                }
                            }
                            // Skip identifiers that are an `attribute`
                            "dollar" => {
                                // The second child of the `dollar` should be ignored
                                if Some(node) == parent_node.child(2) {
                                    continue 'matches;
                                }
                            }
                            // Skip identifiers that are the `name` of a for loop, or that refer to it
                            "for" => {
                                if let Some(name) = parent_node.child_by_field_name("name") {
                                    if name == node || name.utf8_text(code).unwrap() == symbol {
                                        continue 'matches;
                                    }
                                }
                            }
                            // Skip package identifiers
                            "call" => {
                                let name = child_text(parent_node, "function", code);
                                if name == "library" || name == "require" {
                                    continue 'matches;
                                }
                            }
                            // Skip identifiers within a function definition
                            "function_definition" => continue 'matches,
                            _ => {}
                        }
                        parent = parent_node.parent();
                    }

                    let kind = match node.parent() {
                        Some(parent_node) => match parent_node.kind() {
                            "call" => {
                                // Because there are so many globals, unlike for other languages
                                // we only ignore apparent uses of global functions in calls.
                                // This avoids false negatives when a variable has been given a
                                // global name (e.g. "data").
                                if USE_IGNORE.contains(&symbol.as_str()) {
                                    continue 'matches;
                                }
                                Some("Function".to_string())
                            }
                            _ => None,
                        },
                        None => None,
                    };

                    dependencies.push(uses_variable(&symbol, path, kind, Some(range)))
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
    fn parse_r_fragments() {
        snapshot_fixtures("fragments/r/*.R", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let parse_info = RParser::parse(&code, Some(path)).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }
}
