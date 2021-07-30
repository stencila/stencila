use super::{remove_quotes, Compiler};
use crate::graphs::{Relation, Resource};
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
pub fn compile(code: &str) -> Vec<(Relation, Resource)> {
    let code = code.as_bytes();
    let tree = COMPILER.parse(code);
    let captures = COMPILER.query(code, &tree);
    captures
        .iter()
        .filter_map(|(pattern, captures)| match pattern {
            0 => Some((
                Relation::Uses,
                Resource::Module(["javascript/", &remove_quotes(&captures[0].text)].concat()),
            )),
            1 => Some((
                Relation::Uses,
                Resource::Module(["javascript/", &remove_quotes(&captures[1].text)].concat()),
            )),
            2 => Some((
                Relation::Reads,
                Resource::File(remove_quotes(&captures[1].text)),
            )),
            3 => Some((
                Relation::Writes,
                Resource::File(remove_quotes(&captures[1].text)),
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[test]
    fn js_fragments() {
        snapshot_content("fragments/js/*.js", |code| {
            assert_json_snapshot!(compile(&code));
        });
    }
}
