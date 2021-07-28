use super::{captures_as_args_map, is_quoted, remove_quotes, Compiler};
use crate::graphs::{Relation, Resource};
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
    arguments: [
        (
            arguments
                .
                value: (string) @arg
        )
        (
            arguments
                name: (identifier) @arg_name
                .
                value: (string) @arg_value
        )
    ]
)

(call
    function: (identifier) @function (#match? @function "^write\.")
    arguments: [
        (
            arguments
                .
                value: (_) @arg
                .
                value: (string) @arg
        )
        (
            arguments
                name: (identifier) @arg_name
                .
                value: (string) @arg_value
        )
    ]
)
"#,
    )
});

/// Compile some R code
pub fn compile(code: &str) -> Vec<(Relation, Resource)> {
    COMPILER
        .query(code)
        .into_iter()
        .filter_map(|(pattern, captures)| match pattern {
            0 => {
                let args = captures_as_args_map(captures);
                args.get("0")
                    .or_else(|| args.get("package"))
                    .and_then(|package| {
                        if let Some(is_char) = args.get("character.only") {
                            if is_char.starts_with('T') && !is_quoted(package) {
                                return None;
                            }
                        } else if is_quoted(package) {
                            return None;
                        }
                        Some((Relation::Uses, Resource::Module(remove_quotes(package))))
                    })
            }
            1 => {
                let args = captures_as_args_map(captures);
                args.get("0")
                    .or_else(|| args.get("file"))
                    .map(|file| (Relation::Reads, Resource::File(remove_quotes(file))))
            }
            2 => {
                let args = captures_as_args_map(captures);
                args.get("1")
                    .or_else(|| args.get("file"))
                    .map(|file| (Relation::Writes, Resource::File(remove_quotes(file))))
            }
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
    fn r_fragments() {
        snapshot_content("fragments/r/*.R", |code| {
            assert_json_snapshot!(compile(&code));
        });
    }
}
