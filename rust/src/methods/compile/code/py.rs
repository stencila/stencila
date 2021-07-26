use super::{
    captures_as_args_map, code_analysis, is_quoted, remove_quotes, CodeAnalysis, Compiler,
};
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

(call
    function: (identifier) @function (#match? @function "^open$")
    arguments: (
        argument_list
            ([(string)(identifier)] @arg)*
            ([(string)(identifier)] @arg)*
            (keyword_argument
                name: (identifier) @arg_name
                value: (_) @arg_value
            )*
            (keyword_argument
                name: (identifier) @arg_name
                value: (_) @arg_value
            )*
    )
)
"#,
    )
});

/// Compile some Python code
pub fn compile(code: &str) -> CodeAnalysis {
    let mut imports_packages: Vec<String> = Vec::new();
    let mut reads_files: Vec<String> = Vec::new();

    for capture in COMPILER.query(code) {
        let (pattern, captures) = capture;
        match pattern {
            0 => imports_packages.push(captures[0].text.clone()),
            1 => imports_packages.push(captures[0].text.clone()),
            2 => {
                let args = captures_as_args_map(captures);
                if let Some(file) = args.get("0").or_else(|| args.get("file")) {
                    if !is_quoted(file) {
                        continue
                    }
                    if let Some(mode) = args.get("1").or_else(|| args.get("mode")) {
                        if !is_quoted(mode) || !remove_quotes(mode).starts_with('r') {
                            continue;
                        }
                    }
                    reads_files.push(remove_quotes(&file))
                }
            }
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
    fn py_fragments() {
        snapshot_content("fragments/py/*.py", |code| {
            assert_json_snapshot!(compile(&code));
        });
    }
}
