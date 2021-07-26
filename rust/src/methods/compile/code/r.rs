use super::{Compiler, SoftwareSourceAnalysis};
use once_cell::sync::Lazy;

/// Compiler for R
static COMPILER: Lazy<Compiler> = Lazy::new(|| {
    Compiler::new(
        tree_sitter_r::language(),
        r#"
(call
    function: (identifier) @function (#match? @function "^library|require$")
    arguments: [
        (
            arguments
                value: [(identifier)(string)] @package
                ((identifier) @argname [(true)(false)] @argval)*
        )
    ]
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
pub fn compile(code: &str) -> SoftwareSourceAnalysis {
    let mut imports_packages: Vec<String> = vec![];
    for capture in COMPILER.query(code) {
        let (pattern, captures) = capture;
        match pattern {
            0 => {
                let package = &captures[1];
                let package = if package.starts_with(&['\"', '\''][..]) {
                    package.replace(&['\"', '\''][..], "")
                } else {
                    // If the package is an identifier, not a string, then `character.only` option
                    // must be false, or unset.
                    let mut ok = true;
                    let mut index = 3;
                    while index < captures.len() {
                        if captures[index - 1] == "character.only"
                            && captures[index].starts_with('T')
                        {
                            ok = false
                        }
                        index += 1;
                    }
                    if !ok {
                        continue;
                    }

                    package.clone()
                };
                imports_packages.push(package)
            }
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
    fn r_fragments() {
        snapshot_content("fragments/r/*.R", |code| {
            assert_json_snapshot!(compile(&code));
        });
    }
}
