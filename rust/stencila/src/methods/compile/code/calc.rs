use super::utils::parse_tags;
use graphs::{relations, resources, Relation, Resource};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Compile some Calc code
pub fn compile(path: &Path, code: &str) -> Vec<(Relation, Resource)> {
    static ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\s*([a-zA-Z_][a-zA-Z_0-9]*)\s*=(.*)").expect("Unable to create regex")
    });

    // Although we could parse the expression part of each line using `fasteval` and looking
    // for `EVar` nodes in the parse tree, it seems that walking the `fasteval`
    // parse tree is not trivial. So, this uses regex to get variable names (avoiding function
    // names)
    static VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(\b[a-zA-Z_][a-zA-Z_0-9]*\b)(\s*\()?").expect("Unable to create regex")
    });

    code.split('\n')
        .enumerate()
        .fold(Vec::new(), |mut pairs, (row, line)| {
            // Skip the line if it is blank
            if line.trim().is_empty() {
                return pairs;
            }

            // Parse any comment
            if line.trim_start().starts_with('#') {
                let (mut relations, _only) =
                    parse_tags(path, "calc", row, line, Some("Number".to_string()));
                pairs.append(&mut relations);
                return pairs;
            }

            // Parse for assignments
            let (start, expr) = if let Some(captures) = ASSIGN_REGEX.captures(line) {
                let symbol = captures.get(1).expect("Should always have group 1");
                let expr = captures.get(2).expect("Should always have group 2");
                pairs.push((
                    relations::assigns((row, symbol.start(), row, symbol.end())),
                    resources::symbol(path, symbol.as_str(), "Number"),
                ));
                (expr.start(), expr.as_str())
            } else {
                (0, line)
            };

            // Parse for uses of variables
            for captures in VAR_REGEX.captures_iter(expr) {
                if captures.get(2).is_none() {
                    let symbol = captures.get(1).expect("Should always have group 1");
                    pairs.push((
                        relations::uses((row, start + symbol.start(), row, start + symbol.end())),
                        resources::symbol(path, symbol.as_str(), "Number"),
                    ))
                }
            }

            pairs
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_fixtures;
    use insta::assert_json_snapshot;
    use std::path::PathBuf;

    #[test]
    fn calc_fragments() {
        snapshot_fixtures("fragments/calc/*.calc", |path, code| {
            assert_json_snapshot!(compile(&PathBuf::from(path), code));
        });
    }
}
