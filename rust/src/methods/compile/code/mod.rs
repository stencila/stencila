///! Functions etc for compiling programming languages in `CodeChunk` and `CodeExpression` nodes.
///!
///! Uses `tree-sitter` to parse source code into a abstract syntax tree which is then used to
///! derive properties of a `CodeAnalysis`.
use crate::graphs::{resources, Range, Relation, Resource};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[cfg(feature = "compile-code-js")]
pub mod js;

#[cfg(feature = "compile-code-py")]
pub mod py;

#[cfg(feature = "compile-code-r")]
pub mod r;

#[cfg(feature = "compile-code-ts")]
pub mod ts;

/// Compile code in a particular language
pub fn compile(path: &Path, code: &str, language: &str) -> Vec<(Relation, Resource)> {
    let pairs = match language {
        #[cfg(feature = "compile-code-js")]
        "js" | "javascript" => js::compile(path, code),

        #[cfg(feature = "compile-code-py")]
        "py" | "python" => py::compile(path, code),

        #[cfg(feature = "compile-code-r")]
        "r" => r::compile(path, code),

        #[cfg(feature = "compile-code-ts")]
        "ts" | "typescript" => ts::compile(path, code),

        _ => Vec::new(),
    };

    // Normalize pairs by removing any `Uses` of locally assigned variables
    let mut normalized: Vec<(Relation, Resource)> = Vec::with_capacity(pairs.len());
    for (relation, object) in pairs {
        let mut include = true;
        if matches!(relation, Relation::Use(..)) {
            for (other_relation, other_object) in &normalized {
                if matches!(other_relation, Relation::Assign(..)) && *other_object == object {
                    include = false;
                    break;
                }
            }
        }
        if !include {
            continue;
        }

        normalized.push((relation, object))
    }
    normalized
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

    /// The captured range
    range: Range,

    /// The captured text
    text: String,
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
    /// - `language`: The `tree-sitter` language definition
    /// - `query`: The `tree-sitter` query definition
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
    /// - `code`: The code to parse
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
            parse_tags(path, lang, comment.range.0, &comment.text);

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

/// Parse a comment into a set of `Relation`/`Resource` pairs and the name relation
/// types for which those specified should be the only relations
fn parse_tags(
    path: &Path,
    lang: &str,
    row: usize,
    comment: &str,
) -> (Vec<(Relation, Resource)>, Vec<String>) {
    static REGEX_TAG: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"@(imports|assigns|uses|modifies|reads|writes)\s+(.*?)(\*/)?$")
            .expect("Unable to create regex")
    });
    static REGEX_ITEMS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\s+|(\s*,\s*)").expect("Unable to create regex"));

    let mut relations: Vec<(Relation, Resource)> = Vec::new();
    let mut only: Vec<String> = Vec::new();
    for (index, line) in comment.lines().enumerate() {
        let range = (row + index, 0, row + index, line.len() - 1);
        if let Some(captures) = REGEX_TAG.captures(line) {
            let tag = captures[1].to_string();
            let relation = match tag.as_str() {
                "imports" => Relation::Use(range),
                "assigns" => Relation::Assign(range),
                "uses" => Relation::Use(range),
                "reads" => Relation::Read(range),
                "writes" => Relation::Write(range),
                _ => continue,
            };

            let items: Vec<String> = REGEX_ITEMS
                .split(captures[2].trim())
                .map(|item| item.to_string())
                .collect();
            for item in items {
                if item == "only" {
                    only.push(tag.clone());
                    continue;
                }

                let resource = match tag.as_str() {
                    "imports" => resources::module(lang, &item),
                    "assigns" | "uses" => resources::symbol(path, &item, ""),
                    "reads" | "writes" => resources::file(&PathBuf::from(item)),
                    _ => continue,
                };
                relations.push((relation.clone(), resource))
            }
        }
    }
    (relations, only)
}
