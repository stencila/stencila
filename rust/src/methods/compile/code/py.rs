use super::{captures_as_args_map, is_quoted, remove_quotes, Compiler};
use crate::graphs::{resources, Relation, Resource};
use itertools::Itertools;
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
                value: (string) @arg_value
            )*
            (keyword_argument
                name: (identifier) @arg_name
                value: (string) @arg_value
            )*
    )
)
"#,
    )
});

/// Compile some Python code
pub fn compile(path: &str, code: &str) -> Vec<(Relation, Resource)> {
    let code = code.as_bytes();
    let tree = COMPILER.parse(code);
    let captures = COMPILER.query(code, &tree);

    captures
        .into_iter()
        .filter_map(|(pattern, captures)| match pattern {
            0 => Some((
                Relation::Uses,
                resources::module("python", &captures[0].text),
            )),
            1 => Some((
                Relation::Uses,
                resources::module("python", &captures[0].text),
            )),
            2 => {
                let args = captures_as_args_map(captures);
                if let Some(file) = args.get("0").or_else(|| args.get("file")) {
                    if !is_quoted(file) {
                        return None;
                    }
                    let path = remove_quotes(file);
                    if let Some(mode) = args.get("1").or_else(|| args.get("mode")) {
                        if !is_quoted(mode) {
                            return None;
                        }
                        let mode = remove_quotes(mode);
                        if mode.starts_with('w') || mode.starts_with('a') {
                            Some((Relation::Writes, resources::file(&path)))
                        } else {
                            Some((Relation::Reads, resources::file(&path)))
                        }
                    } else {
                        Some((Relation::Reads, resources::file(&path)))
                    }
                } else {
                    None
                }
            }
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
    fn py_fragments() {
        snapshot_content("fragments/py/*.py", |path, code| {
            assert_json_snapshot!(compile(path, code));
        });
    }
}
