//! Helper functions for compiling code using `tree-sitter` and converting
//! the resulting parse trees into Stencila graph `Relation`s and `Resource`s.

use graphs::{Range, Relation, Resource};
use std::{collections::HashMap, path::Path, sync::Mutex};
use tree_sitter::{Language, Parser, Query, QueryCursor};

use super::utils::parse_tags;

/// A capture resulting from a `tree-sitter` query
pub(crate) struct Capture<'tree> {
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
pub(crate) fn captures_as_args_map<'tree>(
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
pub(crate) fn child_text<'tree>(
    node: tree_sitter::Node<'tree>,
    name: &str,
    code: &'tree [u8],
) -> &'tree str {
    node.child_by_field_name(name)
        .and_then(|child| child.utf8_text(code).ok())
        .unwrap_or("")
}

/// A compiler for a particular language
///
/// This simply encapsulates a `tree-sitter` usage pattern to
/// avoid repetitive boiler plate code in the language-specific sub-modules.
pub(crate) struct Compiler {
    /// The `tree-sitter` parser
    parser: Mutex<Parser>,

    /// The `tree-sitter` query
    query: Query,
}

impl Compiler {
    /// Create a new compiler for a language with a pre-defined query
    ///
    /// # Arguments
    ///
    /// - `language`: The `tree-sitter` language definition
    /// - `query`: The `tree-sitter` query definition
    pub fn new(language: Language, query: &str) -> Compiler {
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Should be able to set language");
        let parser = Mutex::new(parser);

        let query = Query::new(language, query).expect("Query should compile");

        Compiler { parser, query }
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
        self.parser
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
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), |node| {
            node.utf8_text(code).unwrap_or_default()
        });

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

/// Apply manual tags (e.g. `@uses` in a comment) to the relations
///
/// # Arguments
///
/// - `path`: The path of the subject code resource
/// - `lang`: The language (used for creating `Resource::Module` variants)
/// - `matches`: The matches from querying the code
/// - `pattern`: The pattern from which to extract tags
/// - `relations`: The relations to update based on tags
///
/// Assumes that the first capture has the text content
/// of the comment.
/// If the tag ends in `only` then all existing relations of that type
/// will be removed from `relations`.
pub(crate) fn apply_tags(
    path: &Path,
    lang: &str,
    matches: Vec<(usize, Vec<Capture>)>,
    pattern: usize,
    mut relations: Vec<(Relation, Resource)>,
) -> Vec<(Relation, Resource)> {
    for (pattern_, captures) in matches {
        if pattern_ != pattern {
            continue;
        }

        // Get the new relations from the comment
        let comment = &captures[0];
        let (mut specified_relations, only_relations) =
            parse_tags(path, lang, comment.range.0, &comment.text, None);

        // Remove existing relations if `only` indicators are present
        for only in only_relations {
            relations.retain(|(relation, resource)| {
                !(matches!(relation, Relation::Use(..))
                    && matches!(resource, Resource::Module(..))
                    && only == "imports"
                    || matches!(relation, Relation::Assign(..)) && only == "assigns"
                    || matches!(relation, Relation::Use(..))
                        && matches!(resource, Resource::Symbol(..))
                        && only == "uses")
            })
        }

        // Add specified relations
        relations.append(&mut specified_relations);
    }
    relations
}
