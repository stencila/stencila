use std::path::Path;

use parser::{
    common::{
        eyre::{eyre, Result},
        once_cell::sync::Lazy,
        regex::{Captures, Regex},
    },
    graph_triples::{Resource, ResourceInfo},
    Parser, ParserTrait,
};

/// A parser for PrQL (pronounce "prequel")
///
/// This current implementation piggybacks on top of the sibling `parser-sql` crate:
/// it converts SQL to PrQL which the `SqlParser` then parses (using `tree-sitter-sql`).
/// An alternative would be to use `prql_compiler::parse` directly and then create
/// relations using the generated AST for the query. That would be more efficient (avoiding
/// two parses, but would be quite a lot of work)
pub struct PrqlParser;

impl ParserTrait for PrqlParser {
    fn spec() -> Parser {
        Parser {
            language: "PrQL".to_string(),
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let sql = prql_compiler::compile(code).map_err(|err| eyre!(err.to_string()))?;

        // For some reason prql_compiler puts a space between $ and the name of a binding e.g. "$ par".
        // This corrects that by reverting to "$par".
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new("\\$ ([a-zA-Z_][a-zA-Z_0-9]*)").expect("Unable to create regex")
        });
        let sql = REGEX.replace_all(&sql, |captures: &Captures| ["$", &captures[1]].concat());

        parser_sql::SqlParser::parse(resource, path, &sql)
    }
}

#[cfg(test)]
mod tests {
    use parser::graph_triples::resources;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    use super::*;

    #[test]
    fn parse_prql_fragments() {
        snapshot_fixtures("fragments/prql/*.prql", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource =
                resources::code(path, "", "SoftwareSourceCode", Some("PRQL".to_string()));
            let resource_info = PrqlParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }
}
