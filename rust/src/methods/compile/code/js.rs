use std::path::Path;

use super::{remove_quotes, Compiler};
use crate::{
    graphs::{resources, Relation, Resource},
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
                Some((Relation::Use, object))
            }
            2 => Some((
                Relation::Read,
                resources::file(&merge(path, remove_quotes(&captures[1].text))),
            )),
            3 => Some((
                Relation::Write,
                resources::file(&merge(path, remove_quotes(&captures[1].text))),
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
    use std::path::PathBuf;

    #[test]
    fn js_fragments() {
        snapshot_content("fragments/js/*.js", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });
    }
}
