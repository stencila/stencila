use super::{CodeAnalysis, Compiler, code_analysis, remove_quotes};
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
    arguments: (arguments (string) @module)
)

(call_expression
    function: [
        (
            (identifier) @function (#match? @function "^readFile")
        )
        (
            member_expression
                object: (identifier)
                property: (property_identifier)  @function (#match? @function "^readFile")
        )
    ]
    arguments: (arguments (string) @module)
)
"#,
    )
});

/// Compile some JavaScript code
pub fn compile(code: &str) -> CodeAnalysis {
    let mut imports_packages: Vec<String> = Vec::new();
    let mut reads_files: Vec<String> = Vec::new();

    for capture in COMPILER.query(code) {
        let (pattern, captures) = capture;
        match pattern {
            0 => imports_packages.push(remove_quotes(&captures[0].text)),
            1 => imports_packages.push(remove_quotes(&captures[1].text)),
            2 => reads_files.push(remove_quotes(&captures[1].text)),
            _ => (),
        }
    }

    code_analysis(imports_packages, reads_files)
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
