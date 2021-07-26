use super::{Compiler, SoftwareSourceAnalysis};
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
"#,
    )
});

/// Compile some JavaScript code
pub fn compile(code: &str) -> SoftwareSourceAnalysis {
    let mut imports_packages: Vec<String> = vec![];
    for capture in COMPILER.query(code) {
        let (pattern, captures) = capture;
        match pattern {
            0 => imports_packages.push(captures[0].clone()),
            1 => imports_packages.push(captures[1].clone()),
            _ => (),
        }
    }

    SoftwareSourceAnalysis { imports_packages }
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
