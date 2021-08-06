use std::path::Path;

use super::{captures_as_args_map, child_text, is_quoted, remove_quotes, Compiler};
use crate::{
    graphs::{resources, Relation, Resource, NULL_RANGE},
    utils::path::merge,
};
use itertools::Itertools;
use once_cell::sync::Lazy;

mod ignores;
use ignores::USE_IGNORE;

/// Compiler for R
static COMPILER: Lazy<Compiler> = Lazy::new(|| {
    Compiler::new(
        tree_sitter_r::language(),
        r#"
(call
    function: (identifier) @function (#match? @function "^library|require$")
    arguments:(
        arguments
            ([(identifier)(string)] @arg)*
            (
                (identifier) @arg_name
                [(true)(false)] @arg_value
            )*
    )
)

(call
    function: (identifier) @function (#match? @function "^read\.")
    arguments: [
        (
            arguments
                .
                value: (string) @arg
        )
        (
            arguments
                name: (identifier) @arg_name
                .
                value: (string) @arg_value
        )
    ]
)

(call
    function: (identifier) @function (#match? @function "^write\.")
    arguments: [
        (
            arguments
                .
                value: (_) @arg
                .
                value: (string) @arg
        )
        (
            arguments
                name: (identifier) @arg_name
                .
                value: (string) @arg_value
        )
    ]
)

(program [
    (left_assignment name: (identifier) @identifer value: (_) @value)
    (equals_assignment name: (identifier) @identifer value: (_) @value)
])
(super_assignment name: (identifier) @identifer  value: (_) @value)

((identifier) @identifer)

"#,
    )
});

/// Compile some R code
pub fn compile(path: &Path, code: &str) -> Vec<(Relation, Resource)> {
    let code = code.as_bytes();
    let tree = COMPILER.parse(code);
    let captures = COMPILER.query(code, &tree);

    captures
        .into_iter()
        .filter_map(|(pattern, captures)| match pattern {
            0 => {
                // Imports a package using `library` or `require`
                let args = captures_as_args_map(captures);
                args.get("0")
                    .or_else(|| args.get("package"))
                    .and_then(|package| {
                        if let Some(is_char) = args.get("character.only") {
                            if is_char.starts_with('T') && !is_quoted(package) {
                                return None;
                            }
                        } else if is_quoted(package) {
                            return None;
                        }
                        Some((
                            Relation::Use(NULL_RANGE),
                            resources::module("r", &remove_quotes(package)),
                        ))
                    })
            }
            1 => {
                // Reads a file
                let args = captures_as_args_map(captures);
                args.get("0").or_else(|| args.get("file")).map(|file| {
                    (
                        Relation::Read(NULL_RANGE),
                        resources::file(&merge(path, remove_quotes(file))),
                    )
                })
            }
            2 => {
                // Writes a file
                let args = captures_as_args_map(captures);
                args.get("1").or_else(|| args.get("file")).map(|file| {
                    (
                        Relation::Write(NULL_RANGE),
                        resources::file(&merge(path, remove_quotes(file))),
                    )
                })
            }
            3 | 4 => {
                // Assigns a symbol at the top level of the module
                let range = captures[0].range;
                let name = captures[0].text.clone();
                let value = captures[1].node;
                let kind = match value.kind() {
                    "true" | "false" => "Boolean",
                    "integer" => "Integer",
                    "float" => "Number",
                    "string" => "String",
                    "function_definition" => "Function",
                    "call" => match child_text(value, "function", code) {
                        "data.frame" | "read.csv" | "read.csv2" | "read.delim" | "read.table" => {
                            "Datatable"
                        }
                        _ => "",
                    },
                    _ => "",
                };
                Some((
                    Relation::Assign(range),
                    resources::symbol(path, &name, kind),
                ))
            }
            5 => {
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
                                    return None;
                                }
                            }
                        }
                        // Skip identifiers that are the `name` of a function call argument
                        "arguments" => {
                            let mut cursor = node.walk();
                            for name in parent_node.children_by_field_name("name", &mut cursor) {
                                if name == node {
                                    return None;
                                }
                            }
                        }
                        // Skip identifiers that are an `attribute`
                        "dollar" => {
                            // The second child of the `dollar` should be ignored
                            if Some(node) == parent_node.child(2) {
                                return None;
                            }
                        }
                        // Skip identifiers that are the `name` of a for loop, or that refer to it
                        "for" => {
                            if let Some(name) = parent_node.child_by_field_name("name") {
                                if name == node || name.utf8_text(code).unwrap() == symbol {
                                    return None;
                                }
                            }
                        }
                        // Skip package identifiers
                        "call" => {
                            let name = child_text(parent_node, "function", code);
                            if name == "library" || name == "require" {
                                return None;
                            }
                        }
                        // Skip identifiers within a function definition
                        "function_definition" => return None,
                        _ => {}
                    }
                    parent = parent_node.parent();
                }

                let resource = match node.parent() {
                    Some(parent_node) => match parent_node.kind() {
                        "call" => {
                            // Because there are so many globals, unlike for other languages
                            // we only ignore apparent uses of global functions in calls.
                            // This avoids false negatives when a variable has been given a
                            // global name (e.g. "data").
                            if USE_IGNORE.contains(&symbol.as_str()) {
                                return None;
                            }
                            resources::symbol(path, &symbol, "Function")
                        }
                        _ => resources::symbol(path, &symbol, ""),
                    },
                    None => resources::symbol(path, &symbol, ""),
                };

                Some((Relation::Use(range), resource))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;
    use std::path::PathBuf;

    #[test]
    fn r_fragments() {
        snapshot_content("fragments/r/*.R", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });
    }
}
