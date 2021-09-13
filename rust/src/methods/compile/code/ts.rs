use super::{
    apply_tags,
    js::{self, handle_patterns},
    Compiler,
};
use crate::graphs::{relations, resources, Relation, Resource};
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
    let matches = COMPILER_TS.query(code, &tree);
    let relations_typed = matches
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
                    relations::assigns(range),
                    resources::symbol(path, &name, kind),
                ))
            }
            _ => None,
        });

    // Query the tree for untyped patterns defined in the JavaScript module
    let matches = COMPILER_JS.query(code, &tree);
    let relations_untyped = matches
        .iter()
        .filter_map(|(pattern, capture)| handle_patterns(path, code, pattern, capture));

    let relations = relations_typed.chain(relations_untyped).collect();
    apply_tags(path, "javascript", matches, 0, relations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_fixtures;
    use insta::assert_json_snapshot;
    use std::path::PathBuf;

    #[test]
    fn ts_fragments() {
        snapshot_fixtures("fragments/ts/*.ts", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });

        // JavaScript fragments should also be compilable by the TypeScript compiler
        snapshot_fixtures("fragments/js/*.js", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });
    }
}
