use super::{remove_quotes, Compiler};
use crate::graphs::{resources, Relation, Resource};
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
"#,
    )
});

/// Compile some JavaScript code
pub fn compile(path: &str, code: &str) -> Vec<(Relation, Resource)> {
    let code = code.as_bytes();
    let tree = COMPILER.parse(code);
    let captures = COMPILER.query(code, &tree);

    captures
        .iter()
        .filter_map(|(pattern, captures)| match pattern {
            0 => Some((
                Relation::Uses,
                resources::module("javascript", &remove_quotes(&captures[0].text)),
            )),
            1 => Some((
                Relation::Uses,
                resources::module("javascript", &remove_quotes(&captures[1].text)),
            )),
            2 => Some((
                Relation::Reads,
                resources::file(&remove_quotes(&captures[1].text)),
            )),
            3 => Some((
                Relation::Writes,
                resources::file(&remove_quotes(&captures[1].text)),
            )),
            _ => None,
        })
        .unique()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[test]
    fn js_fragments() {
        snapshot_content("fragments/js/*.js", |path, code| {
            assert_json_snapshot!(compile(path, code));
        });
    }
}
