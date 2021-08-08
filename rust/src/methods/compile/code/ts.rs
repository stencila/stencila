use super::{
    js::{self, handle},
    Compiler,
};
use crate::{
    graphs::{resources, Relation, Resource},
};
use once_cell::sync::Lazy;
use std::path::Path;

/// Compiler for TypeScript
static COMPILER_TS: Lazy<Compiler> = Lazy::new(|| {
    Compiler::new(
        tree_sitter_typescript::language_typescript(),
        // These are query patterns that extend those for JavaScript only
        r#"
(program
    [
        (variable_declaration
            (variable_declarator
                name: (identifier) @name
                type: (type_annotation) @type
            )
        )
        (lexical_declaration
            (variable_declarator
                name: (identifier) @name
                type: (type_annotation) @type
            )
        )
        (export_statement
            declaration: (lexical_declaration
                (variable_declarator
                    name: (identifier) @name
                    type: (type_annotation) @type
                )
            )
        )
    ]
)
"#,
    )
});

/// Compiler that reuses JavaScript query patterns
static COMPILER_JS: Lazy<Compiler> =
    Lazy::new(|| Compiler::new(tree_sitter_typescript::language_typescript(), js::PATTERNS));

/// Compile some TypeScript code
pub fn compile(path: &Path, code: &str) -> Vec<(Relation, Resource)> {
    let code = code.as_bytes();
    let tree = COMPILER_TS.parse(code);

    // Query the tree for typed patterns defined in this module
    let captures = COMPILER_TS.query(code, &tree);
    let relations_typed = captures
        .iter()
        .filter_map(|(pattern, captures)| match pattern {
            0 => {
                // Assigns a symbol at the top level of the module
                let range = captures[0].range;
                let name = captures[0].text.clone();
                let type_annotation = captures[1].node;
                let type_string = type_annotation
                    .named_child(0)
                    .and_then(|node| node.utf8_text(code).ok())
                    .unwrap_or_default();
                let kind = match type_string {
                    "boolean" => "Boolean",
                    "number" => "Number",
                    "string" => "String",
                    "object" => "Object",
                    _ => {
                        if type_string.starts_with("Array<") {
                            "Array"
                        } else if type_string.starts_with("Record<string") {
                            "Object"
                        } else {
                            ""
                        }
                    }
                };
                Some((
                    Relation::Assign(range),
                    resources::symbol(path, &name, kind),
                ))
            }
            _ => None,
        });

    // Query the tree for untyped patterns defined in the JavaScript module
    let captures = COMPILER_JS.query(code, &tree);
    let relations_untyped = captures
        .iter()
        .filter_map(|(pattern, capture)| handle(path, code, pattern, capture));

    relations_typed.chain(relations_untyped).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;
    use std::path::PathBuf;

    #[test]
    fn ts_fragments() {
        snapshot_content("fragments/ts/*.ts", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });

        // JavaScript fragments should also be compilable by the TypeScript compiler
        snapshot_content("fragments/js/*.js", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });
    }
}
