use once_cell::sync::Lazy;
use parser_treesitter::{
    eyre::Result,
    formats::Format,
    graph_triples::{relations, resources},
    parse_info, ParseInfo, Parser, ParserTrait, TreesitterParser,
};
use std::path::Path;

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
const QUERY: &str = r#"
(program
    [
        (variable_declaration
            (variable_declarator
                name: (identifier) @name
                type: (type_annotation) @type
            )
        )
        (lexical_declaration
            (variable_declarator
                name: (identifier) @name
                type: (type_annotation) @type
            )
        )
        (export_statement
            declaration: (lexical_declaration
                (variable_declarator
                    name: (identifier) @name
                    type: (type_annotation) @type
                )
            )
        )
    ]
)
"#;

/// A parser for TypeScript
pub struct TsParser {}

impl ParserTrait for TsParser {
    fn spec() -> Parser {
        Parser {
            language: Format::TypeScript.spec().title,
        }
    }

    fn parse(path: &Path, code: &str) -> Result<ParseInfo> {
        let code = code.as_bytes();
        let tree = PARSER_TS.parse(code);

        // Query the tree for typed patterns defined in this module
        let matches = PARSER_TS.query(code, &tree);
        let relations_typed = matches
            .iter()
            .filter_map(|(pattern, captures)| match pattern {
                0 => {
                    // Assigns a symbol at the top level of the module
                    let range = captures[0].range;
                    let name = captures[0].text.clone();
                    let type_annotation = captures[1].node;
                    let type_string = type_annotation
                        .named_child(0)
                        .and_then(|node| node.utf8_text(code).ok())
                        .unwrap_or_default();
                    let kind = match type_string {
                        "boolean" => "Boolean",
                        "number" => "Number",
                        "string" => "String",
                        "object" => "Object",
                        _ => {
                            if type_string.starts_with("Array<") {
                                "Array"
                            } else if type_string.starts_with("Record<string") {
                                "Object"
                            } else {
                                ""
                            }
                        }
                    };
                    Some((
                        relations::assigns(range),
                        resources::symbol(path, &name, kind),
                    ))
                }
                _ => None,
            });

        // Query the tree for untyped patterns defined in the JavaScript module
        let matches = PARSER_JS.query(code, &tree);
        let relations_untyped = matches.iter().filter_map(|(pattern, capture)| {
            parser_js::handle_patterns(path, code, pattern, capture)
        });

        let relations = relations_typed.chain(relations_untyped).collect();

        let parse_info = parse_info(path, &Self::spec().language, code, matches, 0, relations);
        Ok(parse_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    #[test]
    fn parse_ts_fragments() {
        snapshot_fixtures("fragments/ts/*.ts", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let parse_info = TsParser::parse(path, &code).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }

    #[test]
    fn parse_js_fragments() {
        snapshot_fixtures("fragments/js/*.js", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let parse_info = TsParser::parse(path, &code).expect("Unable to parse");
            assert_json_snapshot!(parse_info);
        })
    }
}
