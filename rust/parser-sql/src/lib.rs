use std::path::Path;

use parser_treesitter::{
    common::{
        eyre::Result,
        once_cell::sync::Lazy,
        regex::{Captures, Regex},
        tracing,
    },
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
        // Replace named bindings e.g. `$par_a` with numeric bindings e.g. `$1` so that
        // they are recognized by `tree-sitter-sql`.
        let mut bindings = Vec::new();
        static BINDING_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\$([a-zA-Z_][a-zA-Z_0-9]*)").expect("Unable to create regex")
        });
        let sql = BINDING_REGEX.replace_all(code, |captures: &Captures| {
            let name = captures[1].to_string();
            bindings.push(name);
            ["$", &bindings.len().to_string()].concat()
        });

        let code = sql.as_bytes();
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
                    4 => match captures[0].text[1..].parse::<usize>() {
                        Ok(index) => &bindings[index - 1],
                        Err(error) => {
                            tracing::error!(
                                "Unexpectedly unable to parse binding as integer index: {}",
                                error
                            );
                            return None;
                        }
                    },
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
