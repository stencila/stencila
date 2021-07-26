use super::{Compiler, SoftwareSourceAnalysis};
use once_cell::sync::Lazy;

/// Compiler for Python
static COMPILER: Lazy<Compiler> = Lazy::new(|| {
    Compiler::new(
        tree_sitter_python::language(),
        r#"
(import_statement
    name: (dotted_name) @module
)

(import_from_statement
    module_name: (dotted_name) @module
)       
"#,
    )
});

/// Compile some Python code
pub fn compile(code: &str) -> SoftwareSourceAnalysis {
    let mut imports_packages: Vec<String> = vec![];
    for capture in COMPILER.query(code) {
        let (pattern, captures) = capture;
        match pattern {
            0 => imports_packages.push(captures[0].clone()),
            1 => imports_packages.push(captures[0].clone()),
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
    fn py_fragments() {
        snapshot_content("fragments/py/*.py", |code| {
            assert_json_snapshot!(compile(&code));
        });
    }
}
