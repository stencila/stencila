use std::{collections::HashMap, sync::Mutex};

///! Functions etc for compiling programming languages in `CodeChunk` and `CodeExpression` nodes.
///!
///! Uses `tree-sitter` to parse source code into a abstract syntax tree which is then used to
///! derive properties of a `CodeAnalysis`.
use defaults::Defaults;
use eyre::{bail, Result};
use serde::Serialize;
use serde_with::skip_serializing_none;
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[cfg(feature = "compile-code-js")]
mod js;

#[cfg(feature = "compile-code-py")]
mod py;

#[cfg(feature = "compile-code-r")]
mod r;

/// The results of a semantic analysis of a `CodeChunk` or `CodeExpression`
#[skip_serializing_none]
#[derive(Clone, Defaults, Serialize)]
pub struct CodeAnalysis {
    /// A list of modules/packages that the code imports
    imports_packages: Option<Vec<String>>,

    /// A list of files that the code reads
    reads_files: Option<Vec<String>>,

    /// A list of files that the code writes
    writes_files: Option<Vec<String>>,
}

fn code_analysis(
    imports_packages: Vec<String>,
    reads_files: Vec<String>,
    writes_files: Vec<String>,
) -> CodeAnalysis {
    let imports_packages = match imports_packages.is_empty() {
        false => Some(imports_packages),
        true => None,
    };

    let reads_files = match reads_files.is_empty() {
        false => Some(reads_files),
        true => None,
    };

    let writes_files = match writes_files.is_empty() {
        false => Some(writes_files),
        true => None,
    };

    CodeAnalysis {
        imports_packages,
        reads_files,
        writes_files,
    }
}

/// Compile code in a particular language
///
/// # Arguments
///
/// - `code`: the code to compile
/// - `language`: the language that the code is in
///
/// # Returns
///
/// A `CodeAnalysis` summarizing the semantic analysis of the code.
pub fn compile(code: &str, language: &str) -> Result<CodeAnalysis> {
    let analysis = match language {
        #[cfg(feature = "compile-code-js")]
        "js" | "javascript" => js::compile(code),
        #[cfg(feature = "compile-code-py")]
        "py" | "python" => py::compile(code),
        #[cfg(feature = "compile-code-r")]
        "r" => r::compile(code),
        _ => bail!("Unable to compile programming language '{}'", language),
    };
    Ok(analysis)
}

pub(crate) struct Capture {
    #[allow(dead_code)]
    /// The index of the capture in the pattern
    index: u32,

    /// The name of the capture in the pattern
    name: String,

    /// The captured text
    text: String,
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

    /// Parse and query some code
    ///
    /// # Arguments
    ///
    /// - `code`: the code to parse
    ///
    /// # Returns
    ///
    /// A vector of `(pattern, captures)` enumerating the matches for
    /// patterns in the query.
    fn query(&self, code: &str) -> Vec<(usize, Vec<Capture>)> {
        let code = code.as_bytes();
        let tree = self
            .parser
            .lock()
            .expect("Unable to lock parser")
            .parse(code, None)
            .expect("Should be a tree result");
        let root = tree.root_node();

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, root, |node| {
            node.utf8_text(code).unwrap_or_default()
        });

        let capture_names = self.query.capture_names();
        matches
            .map(|query_match| {
                let pattern = query_match.pattern_index;
                let captures = query_match
                    .captures
                    .iter()
                    .map(|capture| Capture {
                        index: capture.index,
                        name: capture_names[capture.index as usize].clone(),
                        text: capture
                            .node
                            .utf8_text(code)
                            .expect("Should be able to get text")
                            .to_string(),
                    })
                    .collect();
                (pattern, captures)
            })
            .collect()
    }
}
