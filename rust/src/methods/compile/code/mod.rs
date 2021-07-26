use std::sync::Mutex;

///! Functions etc for compiling programming languages in `CodeChunk` and `CodeExpression` nodes.
///!
///! Uses `tree-sitter` to parse source code into a abstract syntax tree which is then used to
///! derive properties of a `SoftwareSourceAnalysis`.
use defaults::Defaults;
use eyre::{bail, Result};
use serde::Serialize;
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[cfg(feature = "compile-code-js")]
mod js;

#[cfg(feature = "compile-code-py")]
mod py;

#[cfg(feature = "compile-code-r")]
mod r;

#[derive(Clone, Defaults, Serialize)]
pub struct SoftwareSourceAnalysis {
    imports_packages: Vec<String>,
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
/// A `SoftwareSourceAnalysis` summarizing the semantic analysis of the code.
pub fn compile(code: &str, language: &str) -> Result<SoftwareSourceAnalysis> {
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
    fn query(&self, code: &str) -> Vec<(usize, Vec<String>)> {
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

        matches
            .map(|query_match| {
                let pattern = query_match.pattern_index;
                let captures: Vec<String> = query_match
                    .captures
                    .iter()
                    .map(|capture| {
                        capture
                            .node
                            .utf8_text(code)
                            .expect("Should be able to get text")
                            .to_string()
                    })
                    .collect();
                (pattern, captures)
            })
            .collect()
    }
}
