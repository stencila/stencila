use once_cell::sync::Lazy;
use parser_treesitter::{
    apply_comment_tags,
    eyre::Result,
    formats::Format,
    graph_triples::{relations, resources, Pair, Pairs},
    path_utils,
    utils::remove_quotes,
    Capture, Parser, ParserTrait, TreesitterParser,
};
use std::path::Path;

/// Tree-sitter based parser for JavaScript
static PARSER: Lazy<TreesitterParser> =
    Lazy::new(|| TreesitterParser::new(tree_sitter_javascript::language(), QUERY));

/// Tree-sitter AST query for JavaScript
///
/// Made public for use by `parser-ts`.
pub const QUERY: &str = r#"
(program (comment) @comment)

(import_statement
    source: (string) @module
)
(call_expression
    function: [(import)(identifier)] @function (#match? @function "^import|require$")
    arguments: (arguments . (string) @module)
)

(call_expression
    function: [
        (
            (identifier) @function (#match? @function "^readFile")
        )
        (
            member_expression
                object: (_)
                property: (property_identifier)  @function (#match? @function "^readFile")
        )
    ]
    arguments: (arguments . (string) @path)
)

(call_expression
    function: [
        (
            (identifier) @function (#match? @function "^writeFile")
        )
        (
            member_expression
                object: (_)
                property: (property_identifier)  @function (#match? @function "^writeFile")
        )
    ]
    arguments: (arguments . (string) @path)
)

(program
    [
        (expression_statement
            (assignment_expression
                left: (identifier) @name
                right: (_) @value
            )
        )
        (variable_declaration
            (variable_declarator
                name: (identifier) @name
                .
                value: (_) @value
            )
        )
        (lexical_declaration
            (variable_declarator
                name: (identifier) @name
                .
                value: (_) @value
            )
        )
        (export_statement
            declaration: (lexical_declaration
                (variable_declarator
                    name: (identifier) @name
                    .
                    value: (_) @value
                )
            )
        )
    ]
)
(program
    [
        (function_declaration
            name: (identifier) @name
        )
        (export_statement
            declaration: (function_declaration
                name: (identifier) @name
            )
        )
    ]
)

((identifier) @identifer)
"#;

/// Handle a pattern match
///
/// Made public for use by `parser-ts`.
pub fn handle_patterns(
    path: &Path,
    code: &[u8],
    pattern: &usize,
    captures: &[Capture],
) -> Option<Pair> {
    match pattern {
        1 | 2 => {
            // Imports a module using `import` or `require`
            let capture = match pattern {
                1 => &captures[0],
                2 => &captures[1],
                _ => unreachable!(),
            };
            let range = capture.range;
            let module = remove_quotes(&capture.text.clone());
            let object = if module.starts_with("./") {
                resources::file(&path_utils::merge(path, &[&module, ".js"].concat()))
            } else {
                resources::module("javascript", &module)
            };
            Some((relations::uses(range), object))
        }
        3 => {
            // Reads a file
            Some((
                relations::reads(captures[1].range),
                resources::file(&path_utils::merge(path, remove_quotes(&captures[1].text))),
            ))
        }
        4 => {
            // Writes a file
            Some((
                relations::writes(captures[1].range),
                resources::file(&path_utils::merge(path, remove_quotes(&captures[1].text))),
            ))
        }
        5 | 6 => {
            // Assigns a symbol at the top level of the module
            let range = captures[0].range;
            let name = captures[0].text.clone();
            let kind = match pattern {
                5 => match captures[1].node.kind() {
                    "true" | "false" => "Boolean",
                    "number" => "Number",
                    "string" => "String",
                    "array" => "Array",
                    "object" => "Object",
                    "arrow_function" => "Function",
                    _ => "",
                },
                6 => "Function",
                _ => unreachable!(),
            };
            Some((
                relations::assigns(range),
                resources::symbol(path, &name, kind),
            ))
        }
        7 => {
            // Uses an identifier assigned elsewhere
            let node = captures[0].node;
            let range = captures[0].range;
            let symbol = captures[0].text.clone();

            let mut parent = node.parent();
            while let Some(parent_node) = parent {
                match parent_node.kind() {
                    // Skip identifiers in import statements
                    // Could just skip children of `import_statement`, but specifying others in tree
                    // results in an earlier return while walking up tree.
                    "import_statement" | "import_clause" | "named_imports" | "import_specifier" => {
                        return None
                    }
                    // Skip identifiers that are the `name` of a declaration
                    "variable_declarator" => {
                        if Some(node) == parent_node.child_by_field_name("name") {
                            return None;
                        }
                    }
                    // Skip identifiers that are the `left` of an assignment
                    "assignment_expression" => {
                        if Some(node) == parent_node.child_by_field_name("left") {
                            return None;
                        }
                    }
                    // Skip any identifier used in a function
                    "function_declaration" | "arrow_function" | "formal_parameters" => {
                        return None;
                    }
                    // Skip identifiers that are the `left` of a for in loop, or that refer to it
                    // within the loop
                    "for_in_statement" => {
                        if let Some(left) = parent_node.child_by_field_name("left") {
                            if left == node || left.utf8_text(code).unwrap() == symbol {
                                return None;
                            }
                        }
                    }
                    // Skip identifiers that are the `name` of the variable in a for loop, or
                    // that refer to it within the loop
                    "for_statement" => {
                        if let Some(name) = parent_node
                            .child_by_field_name("initializer")
                            .and_then(|node| node.named_child(0)) // variable_declarator
                            .and_then(|node| node.child_by_field_name("name"))
                        {
                            if name == node || name.utf8_text(code).unwrap() == symbol {
                                return None;
                            }
                        }
                    }
                    _ => {}
                }
                parent = parent_node.parent();
            }

            Some((relations::uses(range), resources::symbol(path, &symbol, "")))
        }
        _ => None,
    }
}

/// A parser for JavaScript
pub struct JsParser {}

impl ParserTrait for JsParser {
    fn spec() -> Parser {
        Parser {
            language: Format::JavaScript.spec().title,
        }
    }

    fn parse(path: &Path, code: &str) -> Result<Pairs> {
        let code = code.as_bytes();
        let tree = PARSER.parse(code);
        let matches = PARSER.query(code, &tree);

        let relations = matches
            .iter()
            .filter_map(|(pattern, capture)| handle_patterns(path, code, pattern, capture))
            .collect();

        let pairs = apply_comment_tags(path, &Self::spec().language, matches, 0, relations);
        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures};
    use test_utils::fixtures;

    #[test]
    fn parse_js_fragments() {
        snapshot_fixtures("fragments/js/*.js", |path| {
            let code = std::fs::read_to_string(path).expect("Unable to read");
            let path = path.strip_prefix(fixtures()).expect("Unable to strip");
            let pairs = JsParser::parse(path, &code).expect("Unable to parse");
            assert_json_snapshot!(pairs);
        })
    }
}
