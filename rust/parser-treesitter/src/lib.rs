use parser::{
    graph_triples::{relations::Range, Pairs},
    utils::apply_tags,
};
use std::{collections::HashMap, path::Path, sync::Mutex};

// Re-exports for the convenience of crates implementing a Tree-sitter
// based parser
pub use parser::*;
pub use path_utils;

/// A parser based on `tree-sitter`
///
/// This simply encapsulates a `tree-sitter` usage pattern to
/// avoid repetitive boiler plate code in the language-specific sub-modules.
pub struct TreesitterParser {
    /// The `tree-sitter` parser
    inner: Mutex<tree_sitter::Parser>,

    /// The `tree-sitter` query
    query: tree_sitter::Query,
}

impl TreesitterParser {
    /// Create a new compiler for a language with a pre-defined query
    ///
    /// # Arguments
    ///
    /// - `language`: The `tree-sitter` language definition
    /// - `query`: The `tree-sitter` query definition
    pub fn new(language: tree_sitter::Language, query: &str) -> TreesitterParser {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(language)
            .expect("Should be able to set language");
        let parser = Mutex::new(parser);

        let query = tree_sitter::Query::new(language, query).expect("Query should compile");

        TreesitterParser {
            inner: parser,
            query,
        }
    }

    /// Parse some code and return a tree
    ///
    /// # Arguments
    ///
    /// - `code`: The code to parse
    ///
    /// # Returns
    ///
    /// The parsed syntax tree.
    pub fn parse(&self, code: &[u8]) -> tree_sitter::Tree {
        self.inner
            .lock()
            .expect("Unable to lock parser")
            .parse(code, None)
            .expect("Should be a tree result")
    }

    /// Query a parse tree
    ///
    /// # Arguments
    ///
    /// - `code`: The code to parse
    /// - `tree`: The `tree-sitter` parse tree
    ///
    /// # Returns
    ///
    /// A vector of `(pattern, captures)` enumerating the matches for
    /// patterns in the query.
    pub fn query<'tree>(
        &self,
        code: &[u8],
        tree: &'tree tree_sitter::Tree,
    ) -> Vec<(usize, Vec<Capture<'tree>>)> {
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), code);

        let capture_names = self.query.capture_names();
        matches
            .map(|query_match| {
                let pattern = query_match.pattern_index;
                let captures = query_match
                    .captures
                    .iter()
                    .map(|capture| {
                        let start = capture.node.start_position();
                        let end = capture.node.end_position();
                        Capture::new(
                            capture.index,
                            capture_names[capture.index as usize].clone(),
                            capture.node,
                            (start.row, start.column, end.row, end.column),
                            capture
                                .node
                                .utf8_text(code)
                                .expect("Should be able to get text")
                                .to_string(),
                        )
                    })
                    .collect();
                (pattern, captures)
            })
            .collect()
    }
}

/// A capture resulting from a `tree-sitter` query
pub struct Capture<'tree> {
    #[allow(dead_code)]
    /// The index of the capture in the pattern
    index: u32,

    /// The name of the capture in the pattern
    name: String,

    /// The captured node
    pub node: tree_sitter::Node<'tree>,

    /// The captured range
    pub range: Range,

    /// The captured text
    pub text: String,
}

impl<'tree> Capture<'tree> {
    pub fn new(
        index: u32,
        name: String,
        node: tree_sitter::Node<'tree>,
        range: Range,
        text: String,
    ) -> Capture {
        Capture {
            index,
            name,
            node,
            range,
            text,
        }
    }
}

/// Convert a vector of `Capture`s into a `HashMap` of captures
///
/// This relies on captures using the names `@arg` (for non-keyword args)
/// and `@arg_name` and `@arg_value` pairs (for keyword args).
pub fn captures_as_args_map<'tree>(
    captures: &'tree [Capture],
) -> HashMap<String, &'tree Capture<'tree>> {
    let mut map = HashMap::new();

    let mut index = 0;
    let mut name = String::new();
    for capture in captures {
        match capture.name.as_str() {
            "arg" => {
                map.insert(index.to_string(), capture);
                index += 1;
            }
            "arg_name" => {
                name = capture.text.clone();
            }
            "arg_value" => {
                map.insert(name.clone(), capture);
            }
            _ => {}
        }
    }

    map
}

/// Get the text of a child node
///
/// Returns an empty string if the child does not exists, or the text could
/// not be obtained
pub fn child_text<'tree>(
    node: tree_sitter::Node<'tree>,
    name: &str,
    code: &'tree [u8],
) -> &'tree str {
    node.child_by_field_name(name)
        .and_then(|child| child.utf8_text(code).ok())
        .unwrap_or("")
}

/// Create a [`ParseInfo`] instance from a Treesitter parse tree and pattern matches
///
/// Applies manual tags (e.g. `@uses`) in a comments to the relations derived from
/// semantic code analysis.
///
/// # Arguments
///
/// - `path`: The path of the subject code resource
/// - `lang`: The language (used for creating `Resource::Module` variants)
/// - `code`: The code that was parsed
/// - `matches`: The matches from querying the code
/// - `comment_pattern`: The index of the pattern from which to extract tags
/// - `relations`: The relation pairs
///
/// Assumes that the first capture has the text content of the comment.
/// If the tag ends in `only` then all existing relations of that type
/// will be removed from `relations`.
pub fn parse_info(
    path: &Path,
    lang: &str,
    code: &[u8],
    matches: Vec<(usize, Vec<Capture>)>,
    comment_pattern: usize,
    relations: Pairs,
) -> ParseInfo {
    let mut parse_info = ParseInfo {
        relations,
        code_hash: ParseInfo::hash(&code),
        ..Default::default()
    };

    for (pattern_, captures) in matches {
        if pattern_ != comment_pattern {
            continue;
        }

        let comment = &captures[0];
        apply_tags(
            path,
            lang,
            comment.range.0,
            &comment.text,
            None,
            &mut parse_info,
        )
    }

    parse_info
}
