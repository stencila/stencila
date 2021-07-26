use super::{
    captures_as_args_map, code_analysis, is_quoted, remove_quotes, CodeAnalysis, Compiler,
};
use once_cell::sync::Lazy;

/// Compiler for R
static COMPILER: Lazy<Compiler> = Lazy::new(|| {
    Compiler::new(
        tree_sitter_r::language(),
        r#"
(call
    function: (identifier) @function (#match? @function "^library|require$")
    arguments:(
        arguments
            ([(identifier)(string)] @arg)*
            (
                (identifier) @arg_name
                [(true)(false)] @arg_value
            )*
    )
)

(call
    function: (identifier) @function (#match? @function "^read\.")
    arguments: (
        arguments value: (string) @path
    )
)
"#,
    )
});

/// Compile some R code
pub fn compile(code: &str) -> CodeAnalysis {
    let mut imports_packages: Vec<String> = Vec::new();
    let mut reads_files: Vec<String> = Vec::new();

    for capture in COMPILER.query(code) {
        let (pattern, captures) = capture;
        match pattern {
            0 => {
                let args = captures_as_args_map(captures);
                println!("{:?}", args);
                if let Some(package) = args.get("0").or_else(|| args.get("package")) {
                    if let Some(is_char) = args.get("character.only") {
                        if is_char.starts_with('T') {
                            if !is_quoted(package) {
                                continue;
                            }
                        }
                    } else {
                        if is_quoted(package) {
                            continue;
                        }
                    }
                    imports_packages.push(remove_quotes(package))
                }
            }
            1 => {
                let path = remove_quotes(&captures[1].text);
                reads_files.push(path)
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
    fn r_fragments() {
        snapshot_content("fragments/r/*.R", |code| {
            assert_json_snapshot!(compile(&code));
        });
    }
}
