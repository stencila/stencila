use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use parser_treesitter::{
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    parse_info, path_utils,
    stencila_schema::{ExecutionDependency, ExecutionDependent},
    utils::{
        assigns_variable, declares_variable, imports_file, imports_module, reads_file,
        remove_quotes, uses_variable, writes_file,
    },
    Capture, ParseInfo, Parser, ParserTrait, TreesitterParser,
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
            language: Format::JavaScript,
        }
    }

    fn parse(code: &str, path: Option<&Path>) -> Result<ParseInfo> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let mut dependencies = Vec::new();
        let mut dependents = Vec::new();

        let path_buf = path
            .map(PathBuf::from)
            .unwrap_or_else(|| current_dir().expect("Should be able to get pwd"));
        for (pattern, captures) in matches.iter() {
            handle_pattern(
                code,
                &path_buf,
                pattern,
                captures,
                &mut dependencies,
                &mut dependents,
            );
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

/// Handle a pattern match
///
/// Made public for use by `parser-ts`.
pub fn handle_pattern(
    code: &[u8],
    path: &PathBuf,
    pattern: &usize,
    captures: &[Capture],
    dependencies: &mut Vec<ExecutionDependency>,
    dependents: &mut Vec<ExecutionDependent>,
) {
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
            if module.starts_with("./") {
                dependencies.push(imports_file(
                    &path_utils::merge(path, &[&module, ".js"].concat()),
                    Some(range),
                ))
            } else {
                dependencies.push(imports_module(&module, Some(range)))
            }
        }
        3 => {
            // Reads a file
            dependencies.push(reads_file(
                &path_utils::merge(path, remove_quotes(&captures[1].text)),
                Some(captures[1].range),
            ))
        }
        4 => {
            // Writes a file
            dependents.push(writes_file(
                &path_utils::merge(path, remove_quotes(&captures[1].text)),
                Some(captures[1].range),
            ))
        }
        5 | 6 => {
            // Declares a symbol at the top level of the module
            let range = captures[0].range;
            let name = captures[0].text.clone();
            let kind = match pattern {
                5 => node_kind(captures[1].node.kind()),
                6 => Some("Function".to_string()),
                _ => unreachable!(),
            };
            dependents.push(declares_variable(&name, Some(path), kind, Some(range)))
        }
        7 => {
            // Assigns a symbol at the top level of the module
            let range = captures[0].range;
            let name = captures[0].text.clone();
            let kind = node_kind(captures[1].node.kind());
            dependents.push(assigns_variable(&name, Some(path), kind, Some(range)))
        }
        8 | 9 => {
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
                        return
                    }
                    // Skip identifiers that are the `name` of a declaration
                    "variable_declarator" => {
                        if Some(node) == parent_node.child_by_field_name("name") {
                            return;
                        }
                    }
                    // Skip identifiers that are the `left` of an assignment
                    "assignment_expression" => {
                        if Some(node) == parent_node.child_by_field_name("left") {
                            return;
                        }
                    }
                    // Skip any identifier used in a function
                    "function_declaration" | "arrow_function" | "formal_parameters" => {
                        return;
                    }
                    // Skip identifiers that are the `left` of a for in loop, or that refer to it
                    // within the loop
                    "for_in_statement" => {
                        if let Some(left) = parent_node.child_by_field_name("left") {
                            if left == node || left.utf8_text(code).unwrap() == symbol {
                                return;
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
                                return;
                            }
                        }
                    }
                    _ => {}
                }
                parent = parent_node.parent();
            }
            dependencies.push(uses_variable(&symbol, Some(path), None, Some(range)));
        }
        _ => (),
    }
}

// Translate a `tree-sitter-javascript` AST node `kind` into a Stencila node `type`
fn node_kind(kind: &str) -> Option<String> {
    match kind {
        "true" | "false" => Some("Boolean".to_string()),
        "number" => Some("Number".to_string()),
        "string" => Some("String".to_string()),
        "array" => Some("Array".to_string()),
        "object" => Some("Object".to_string()),
        "arrow_function" => Some("Function".to_string()),
        _ => None,
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
            let parse_info = JsParser::parse(&code, Some(path)).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }
}
