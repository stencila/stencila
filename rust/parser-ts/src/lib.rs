use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use parser_treesitter::{
    common::{eyre::Result, once_cell::sync::Lazy},
    formats::Format,
    parse_info,
    utils::declares_variable,
    ParseInfo, Parser, ParserTrait, TreesitterParser,
};

/// Tree-sitter based parser for JavaScript
static PARSER_JS: Lazy<TreesitterParser> = Lazy::new(|| {
    TreesitterParser::new(
        tree_sitter_typescript::language_typescript(),
        parser_js::QUERY,
    )
});

/// Tree-sitter based parser for TypeScript
static PARSER_TS: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_typescript::language_typescript(), QUERY));

/// Tree-sitter AST query for TypeScript
///
/// These are query patterns that extend those for JavaScript defined
/// in `parser-js`.
const QUERY: &str = include_str!("query.scm");

/// A parser for TypeScript
pub struct TsParser {}

impl ParserTrait for TsParser {
    fn spec() -> Parser {
        Parser {
            language: Format::TypeScript,
        }
    }

    fn parse(code: &str, path: Option<&Path>) -> Result<ParseInfo> {
        let code = code.as_bytes();
        let tree = PARSER_TS.parse(code);

        let mut dependencies = Vec::new();
        let mut dependents = Vec::new();

        // Query the tree for typed patterns defined in this module
        let matches = PARSER_TS.query(code, &tree);
        for (pattern, captures) in matches.iter() {
            if *pattern == 0 {
                // Assigns a symbol at the top level of the module
                let range = captures[0].range;
                let name = captures[0].text.clone();
                let type_annotation = captures[1].node;
                let type_string = type_annotation
                    .named_child(0)
                    .and_then(|node| node.utf8_text(code).ok())
                    .unwrap_or_default();
                let kind = match type_string {
                    "boolean" => Some("Boolean".to_string()),
                    "number" => Some("Number".to_string()),
                    "string" => Some("String".to_string()),
                    "object" => Some("Object".to_string()),
                    _ => {
                        if type_string.starts_with("Array<") {
                            Some("Array".to_string())
                        } else if type_string.starts_with("Record<string") {
                            Some("Object".to_string())
                        } else {
                            None
                        }
                    }
                };
                dependents.push(declares_variable(&name, path, kind, Some(range)))
            }
        }

        // Query the tree for untyped patterns defined in the JavaScript module
        let matches = PARSER_JS.query(code, &tree);
        let path_buf = path
            .map(PathBuf::from)
            .unwrap_or_else(|| current_dir().expect("Should be able to get pwd"));
        for (pattern, captures) in matches.iter() {
            parser_js::handle_pattern(
                code,
                &path_buf,
                pattern,
                captures,
                &mut dependencies,
                &mut dependents,
            )
        }

        let parse_info = parse_info(
            path,
            Self::spec().language,
            code,
            &tree,
            &["comment"],
            matches,
            0,
            dependencies,
            dependents,
        );
        Ok(parse_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::fixtures;
    use test_utils::{insta::assert_json_snapshot, snapshot_fixtures};

    #[test]
    fn parse_ts_fragments() {
        snapshot_fixtures("fragments/ts/*.ts", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let parse_info = TsParser::parse(&code, Some(path)).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }

    #[test]
    fn parse_js_fragments() {
        snapshot_fixtures("fragments/js/*.js", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let parse_info = TsParser::parse(&code, Some(path)).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }
}
