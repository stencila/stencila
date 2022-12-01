use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use parser_treesitter::{
    captures_as_args_map,
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    parse_info, path_utils,
    utils::{
        assigns_variable, declares_variable, imports_file, imports_module, is_quoted, reads_file,
        remove_quotes, uses_variable, writes_file,
    },
    ParseInfo, Parser, ParserTrait, TreesitterParser,
};

/// Tree-sitter based parser for Python
static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_python::language(), QUERY));

/// Tree-sitter AST query
const QUERY: &str = include_str!("query.scm");

mod ignores;
use ignores::USE_IGNORE;

/// A parser for Python
pub struct PythonParser {}

impl ParserTrait for PythonParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Python,
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
                1 | 2 => {
                    // Imports a module
                    let range = captures[0].range;
                    let module = &captures[0].text;
                    let file_path = path_utils::merge(
                        path.map(PathBuf::from)
                            .unwrap_or_else(|| current_dir().expect("Should be able to get pwd")),
                        [module, ".py"].concat(),
                    );
                    match file_path.exists() {
                        true => dependencies.push(imports_file(&file_path, Some(range))),
                        false => dependencies.push(imports_module(module, Some(range))),
                    };
                }
                3 => {
                    // Opens a file for reading or writing
                    let args = captures_as_args_map(captures);
                    if let Some(file) = args.get("0").or_else(|| args.get("file")) {
                        if !is_quoted(&file.text) {
                            continue;
                        }
                        let file_path = path_utils::merge(
                            path.map(PathBuf::from).unwrap_or_else(|| {
                                current_dir().expect("Should be able to get pwd")
                            }),
                            remove_quotes(&file.text),
                        );
                        let range = file.range;
                        if let Some(mode) = args.get("1").or_else(|| args.get("mode")) {
                            if !is_quoted(&mode.text) {
                                continue;
                            }
                            let mode = remove_quotes(&mode.text);
                            if mode.starts_with('w') || mode.starts_with('a') {
                                dependents.push(writes_file(&file_path, Some(range)))
                            } else {
                                dependencies.push(reads_file(&file_path, Some(range)))
                            }
                        } else {
                            dependencies.push(reads_file(&file_path, Some(range)))
                        }
                    }
                }
                4 | 5 => {
                    // Assigns a symbol at the top level of the module
                    let range = captures[0].range;
                    let name = captures[0].text.clone();
                    let kind = match pattern {
                        4 => match captures[1].node.kind() {
                            "true" | "false" => Some("Boolean".to_string()),
                            "integer" => Some("Integer".to_string()),
                            "float" => Some("Number".to_string()),
                            "string" => Some("String".to_string()),
                            "list" => Some("Array".to_string()),
                            "dictionary" => Some("Object".to_string()),
                            "lambda" => Some("Function".to_string()),
                            _ => None,
                        },
                        5 => Some("Function".to_string()),
                        _ => unreachable!(),
                    };

                    match pattern {
                        4 => dependents.push(assigns_variable(&name, path, kind, Some(range))),
                        5 => dependents.push(declares_variable(&name, path, kind, Some(range))),
                        _ => unreachable!(),
                    };
                }
                6 => {
                    // Uses an identifier assigned elsewhere
                    let node = captures[0].node;
                    let range = captures[0].range;
                    let symbol = captures[0].text.clone();

                    if USE_IGNORE.contains(&symbol.as_str()) {
                        continue;
                    }

                    let mut parent = node.parent();
                    while let Some(parent_node) = parent {
                        match parent_node.kind() {
                            // Skip identifiers that are the `left` of an assignment
                            "assignment" => {
                                if Some(node) == parent_node.child_by_field_name("left") {
                                    continue 'matches;
                                }
                            }
                            // Skip any identifier used in a function parameter
                            "parameters" | "lambda_parameters" => continue 'matches,
                            // Skip identifiers that are the `name` of a keyword argument
                            "keyword_argument" => {
                                if Some(node) == parent_node.child_by_field_name("name") {
                                    continue 'matches;
                                }
                            }
                            // Skip identifiers that are an `attribute`
                            "object" | "function" | "attribute" => {
                                if Some(node) == parent_node.child_by_field_name("attribute") {
                                    continue 'matches;
                                }
                            }
                            // Skip identifiers that are the `left` of a for loop, or that refer to it
                            // within the loop
                            "for_statement" => {
                                if let Some(left) = parent_node.child_by_field_name("left") {
                                    if left == node || left.utf8_text(code).unwrap() == symbol {
                                        continue 'matches;
                                    }
                                }
                            }
                            // Skip identifiers within these...
                            "import_statement"
                            | "import_from_statement"
                            | "function_definition"
                            | "lambda" => continue 'matches,
                            // Skip identifiers that are the identifier in an `as_pattern_target`
                            "as_pattern_target" => continue 'matches,
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
                                                continue 'matches;
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

                    dependencies.push(uses_variable(&symbol, path, None, Some(range)))
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
    use test_utils::fixtures;
    use test_utils::{insta::assert_json_snapshot, snapshot_fixtures};

    #[test]
    fn parse_py_fragments() {
        snapshot_fixtures("fragments/py/*.py", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let parse_info = PythonParser::parse(&code, Some(path)).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }
}
