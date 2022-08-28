use std::path::Path;

use parser_treesitter::{
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    graph_triples::{relations, resources, Resource, ResourceInfo},
    resource_info, Parser, ParserTrait, TreesitterParser,
};

/// A parser for SQL
pub struct SqlParser;

static LANG: Lazy<String> = Lazy::new(|| Format::SQL.spec().title);

const QUERY: &str = include_str!("query.scm");

static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_sql::language(), QUERY));

impl ParserTrait for SqlParser {
    fn spec() -> Parser {
        Parser {
            language: LANG.clone(),
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let relations = matches
            .iter()
            .filter_map(|(pattern, captures)| {
                let relation = match pattern {
                    1 => relations::assigns(captures[0].range),
                    2 => relations::uses(captures[0].range),
                    3 => relations::alters(captures[0].range),
                    4 => relations::uses(captures[0].range),
                    _ => return None,
                };
                let name = match pattern {
                    // Note: although tree-sitter-sql creates an ERROR here because of a non-numeric
                    // reference we are able to capture the name of the argument from the reference
                    4 => &captures[0].text[1..],
                    _ => &captures[0].text,
                };
                let kind = match pattern {
                    4 => "",
                    _ => "Datatable",
                };
                Some((relation, resources::symbol(path, name, kind)))
            })
            .collect();

        let resource_info = resource_info(
            resource,
            path,
            &Self::spec().language,
            code,
            &tree,
            &["comment"],
            matches,
            0,
            relations,
        );
        Ok(resource_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    #[test]
    fn parse_sql_fragments() {
        snapshot_fixtures("fragments/sql/*.sql", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let resource = resources::code(path, "", "SoftwareSourceCode", Some("SQL".to_string()));
            let resource_info = SqlParser::parse(resource, path, &code).expect("Unable to parse");
            assert_json_snapshot!(resource_info);
        })
    }
}
