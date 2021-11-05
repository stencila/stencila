use once_cell::sync::Lazy;
use parser::{
    eyre::Result,
    graph_triples::{relations, resources, Pairs},
    utils::parse_tags,
    Parser, ParserTrait,
};
use regex::Regex;
use std::path::Path;

/// A parser for the "Calc" language
struct CalcParser {}

impl ParserTrait for CalcParser {
    fn spec() -> Parser {
        Parser {
            language: "calc".to_string(),
        }
    }

    fn parse(path: &Path, code: &str) -> Result<Pairs> {
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

        let pairs = code
            .split('\n')
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
                            relations::uses((
                                row,
                                start + symbol.start(),
                                row,
                                start + symbol.end(),
                            )),
                            resources::symbol(path, symbol.as_str(), "Number"),
                        ))
                    }
                }

                pairs
            });

        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    #[test]
    fn calc_fragments() {
        snapshot_fixtures("fragments/calc/*.calc", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let pairs = CalcParser::parse(path, &code).expect("Unable to parse");
            assert_json_snapshot!(pairs);
        })
    }
}
