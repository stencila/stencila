use super::{apply_tags, captures_as_args_map, is_quoted, remove_quotes, Compiler};
use crate::{
    graphs::{relations, resources, Relation, Resource},
    utils::path::merge,
};
use once_cell::sync::Lazy;
use std::path::Path;

mod ignores;
use ignores::USE_IGNORE;

/// Compiler for Python
static COMPILER: Lazy<Compiler> = Lazy::new(|| {
    Compiler::new(
        tree_sitter_python::language(),
        r#"
(module
    [
        (expression_statement (string) @comment)
        (comment) @comment
    ]
)
        
(import_statement
    name: (dotted_name) @module
)
(import_from_statement
    module_name: (dotted_name) @module
)

(call
    function: (identifier) @function (#match? @function "^open$")
    arguments: (
        argument_list
            ([(string)(identifier)] @arg)*
            ([(string)(identifier)] @arg)*
            (keyword_argument
                name: (identifier) @arg_name
                value: (string) @arg_value
            )*
            (keyword_argument
                name: (identifier) @arg_name
                value: (string) @arg_value
            )*
    )
)

(module
    (expression_statement
        (assignment
            left: (identifier) @name
            right: (_) @value
        )
    )
) 
(module
    (function_definition
      name: (identifier) @name
    )
)

((identifier) @identifer)
"#,
    )
});

/// Compile some Python code
pub fn compile(path: &Path, code: &str) -> Vec<(Relation, Resource)> {
    let code = code.as_bytes();
    let tree = COMPILER.parse(code);
    let matches = COMPILER.query(code, &tree);

    let relations = matches
        .iter()
        .filter_map(|(pattern, captures)| match pattern {
            1 | 2 => {
                // Imports a module
                let range = captures[0].range;
                let module = &captures[0].text;
                let path = merge(path, [module, ".py"].concat());
                let object = match path.exists() {
                    true => resources::file(&path),
                    false => resources::module("python", module),
                };
                Some((relations::uses(range), object))
            }
            3 => {
                // Opens a file for reading or writing
                let args = captures_as_args_map(captures);
                if let Some(file) = args.get("0").or_else(|| args.get("file")) {
                    if !is_quoted(&file.text) {
                        return None;
                    }
                    let path = merge(path, remove_quotes(&file.text));
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
                Some((
                    relations::assigns(range),
                    resources::symbol(path, &name, kind),
                ))
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
                        // Skip identifiers that are the `alias` of a with clause
                        "with_clause" => {
                            if Some(node) == parent_node.child_by_field_name("alias") {
                                return None;
                            }
                        }
                        // Skip references to the `alias` of a with clause
                        "with_statement" => {
                            if let Some(alias) = parent_node
                                .child(1) // "with_clause"
                                .and_then(|node| node.child(0)) // "with_item"
                                .and_then(|node| node.child_by_field_name("alias"))
                                .and_then(|node| node.utf8_text(code).ok())
                            {
                                if symbol == alias {
                                    return None;
                                }
                            }
                        }
                        // Skip identifiers within these...
                        "import_statement"
                        | "import_from_statement"
                        | "function_definition"
                        | "lambda" => return None,
                        _ => {}
                    }
                    parent = parent_node.parent();
                }

                Some((relations::uses(range), resources::symbol(path, &symbol, "")))
            }
            _ => None,
        })
        .collect();

    apply_tags(path, "python", matches, 0, relations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_fixtures;
    use insta::assert_json_snapshot;
    use std::path::PathBuf;

    #[test]
    fn py_fragments() {
        snapshot_fixtures("fragments/py/*.py", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });
    }
}
