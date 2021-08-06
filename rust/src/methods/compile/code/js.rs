use std::path::Path;

use super::{remove_quotes, Compiler};
use crate::{
    graphs::{resources, Relation, Resource, NULL_RANGE},
    utils::path::merge,
};
use itertools::Itertools;
use once_cell::sync::Lazy;

/// Compiler for JavaScript
static COMPILER: Lazy<Compiler> = Lazy::new(|| {
    Compiler::new(
        tree_sitter_javascript::language(),
        r#"
(import_statement
    source: (string) @module
)
(call_expression
    function: [(import)(identifier)] @function (#match? @function "^import|require$")
    arguments: (arguments . (string) @module)
)

(call_expression
    function: [
        (
            (identifier) @function (#match? @function "^readFile")
        )
        (
            member_expression
                object: (_)
                property: (property_identifier)  @function (#match? @function "^readFile")
        )
    ]
    arguments: (arguments . (string) @path)
)

(call_expression
    function: [
        (
            (identifier) @function (#match? @function "^writeFile")
        )
        (
            member_expression
                object: (_)
                property: (property_identifier)  @function (#match? @function "^writeFile")
        )
    ]
    arguments: (arguments . (string) @path)
)

(program
    [
        (expression_statement
            (assignment_expression
                left: (identifier) @name
                right: (_) @value
            )
        )
        (variable_declaration
            (variable_declarator
                name: (identifier) @name
                value: (_) @value
            )
        )
        (lexical_declaration
            (variable_declarator
                name: (identifier) @name
                value: (_) @value
            )
        )
        (export_statement
            declaration: (lexical_declaration
                (variable_declarator
                    name: (identifier) @name
                    value: (_) @value
                )
            )
        )
    ]
)
(program
    [
        (function_declaration
            name: (identifier) @name
        )
        (export_statement
            declaration: (function_declaration
                name: (identifier) @name
            )
        )
    ]
)
"#,
    )
});

/// Compile some JavaScript code
pub fn compile(path: &Path, code: &str) -> Vec<(Relation, Resource)> {
    let code = code.as_bytes();
    let tree = COMPILER.parse(code);
    let captures = COMPILER.query(code, &tree);

    captures
        .iter()
        .filter_map(|(pattern, captures)| match pattern {
            0 | 1 => {
                // Imports a module using `import` or `require`
                let module = match pattern {
                    0 => &captures[0],
                    1 => &captures[1],
                    _ => unreachable!(),
                }
                .text
                .clone();
                let module = remove_quotes(&module);
                let object = if module.starts_with("./") {
                    resources::file(&merge(path, &[&module, ".js"].concat()))
                } else {
                    resources::module("javascript", &module)
                };
                Some((Relation::Use(NULL_RANGE), object))
            }
            2 => {
                // Reads a file
                Some((
                    Relation::Read(NULL_RANGE),
                    resources::file(&merge(path, remove_quotes(&captures[1].text))),
                ))
            }
            3 => {
                // Writes a file
                Some((
                    Relation::Write(NULL_RANGE),
                    resources::file(&merge(path, remove_quotes(&captures[1].text))),
                ))
            }
            4 | 5 => {
                // Assigns a symbol at the top level of the module
                let name = captures[0].text.clone();
                let kind = match pattern {
                    4 => match captures[1].node.kind() {
                        "true" | "false" => "Boolean",
                        "number" => "Number",
                        "string" => "String",
                        "array" => "Array",
                        "object" => "Object",
                        "arrow_function" => "Function",
                        _ => "",
                    },
                    5 => "Function",
                    _ => unreachable!(),
                };
                Some((
                    Relation::Assign(NULL_RANGE),
                    resources::symbol(path, &name, kind),
                ))
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
    fn js_fragments() {
        snapshot_content("fragments/js/*.js", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });
    }
}
