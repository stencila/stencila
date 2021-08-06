///! Functions etc for compiling programming languages in `CodeChunk` and `CodeExpression` nodes.
///!
///! Uses `tree-sitter` to parse source code into a abstract syntax tree which is then used to
///! derive properties of a `CodeAnalysis`.
use crate::graphs::{Relation, Resource, Triple};
use std::{collections::HashMap, path::Path, sync::Mutex};
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[cfg(feature = "compile-code-js")]
mod js;

#[cfg(feature = "compile-code-py")]
mod py;

#[cfg(feature = "compile-code-r")]
mod r;

/// Compile code in a particular language
pub fn compile(path: &Path, subject: &Resource, code: &str, language: &str) -> Vec<Triple> {
    let pairs = match language {
        #[cfg(feature = "compile-code-js")]
        "js" | "javascript" => js::compile(path, code),
        #[cfg(feature = "compile-code-py")]
        "py" | "python" => py::compile(path, code),
        #[cfg(feature = "compile-code-r")]
        "r" => r::compile(path, code),
        _ => Vec::new(),
    };

    // Translate pairs into triples and remove any `Uses` of locally assigned variables
    let mut triples = Vec::with_capacity(pairs.len());
    for (relation, object) in pairs {
        if matches!(relation, Relation::Use)
            && triples.contains(&(subject.clone(), Relation::Assign, object.clone()))
        {
            continue;
        }

        let triple = (subject.clone(), relation, object);
        triples.push(triple)
    }
    triples
}

/// A capture resulting from a `tree-sitter` query
pub(crate) struct Capture<'tree> {
    #[allow(dead_code)]
    /// The index of the capture in the pattern
    index: u32,

    /// The name of the capture in the pattern
    name: String,

    /// The captured node
    node: tree_sitter::Node<'tree>,

    /// The captured text
    text: String,
}

impl<'tree> Capture<'tree> {
    pub fn new(index: u32, name: String, node: tree_sitter::Node<'tree>, text: String) -> Capture {
        Capture {
            index,
            name,
            node,
            text,
        }
    }
}

/// Convert a vector of `Capture`s into a
pub(crate) fn captures_as_args_map(captures: Vec<Capture>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    let mut index = 0;
    let mut name = String::new();
    for capture in captures {
        match capture.name.as_str() {
            "arg" => {
                map.insert(index.to_string(), capture.text);
                index += 1;
            }
            "arg_name" => {
                name = capture.text;
            }
            "arg_value" => {
                map.insert(name.clone(), capture.text);
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

/// Whether or not the text is quoted
pub(crate) fn is_quoted(text: &str) -> bool {
    (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
}

/// Remove single and double quotes from text
///
/// Useful for "unquoting" captured string literals.
pub(crate) fn remove_quotes(text: &str) -> String {
    text.replace(&['\"', '\''][..], "")
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
    /// - `language`: the `tree-sitter` language definition
    /// - `query`: the `tree-sitter` query definition
    fn new(language: Language, query: &str) -> Compiler {
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
    /// - `code`: the code to parse
    ///
    /// # Returns
    ///
    /// The parsed syntax tree.
    fn parse(&self, code: &[u8]) -> tree_sitter::Tree {
        self.parser
            .lock()
            .expect("Unable to lock parser")
            .parse(code, None)
            .expect("Should be a tree result")
    }

    /// Query a
    ///
    /// # Arguments
    ///
    /// - `code`: the code to parse
    ///
    /// # Returns
    ///
    /// A vector of `(pattern, captures)` enumerating the matches for
    /// patterns in the query.
    fn query<'tree>(
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
                        Capture::new(
                            capture.index,
                            capture_names[capture.index as usize].clone(),
                            capture.node,
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
