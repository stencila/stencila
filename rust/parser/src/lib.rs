use std::path::Path;

use common::{
    eyre::Result,
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
};
use formats::Format;
use stencila_schema::{
    ExecutionAuto, ExecutionDependency, ExecutionDependencyNode, ExecutionDependent, ExecutionTag,
    Variable, ExecutionDependentNode,
};

mod files;
pub use files::*;
mod tags;
pub use tags::*;
mod variables;
pub use variables::*;

// Export and re-export for the convenience of crates that implement a parser
pub mod utils;
pub use common;
pub use formats;
pub use hash_utils;
pub use stencila_schema;

/// A specification for parsers
///
/// All parsers, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Parser` instance from the
/// `spec` function of `ParserTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", crate = "common::serde")]
pub struct Parser {
    /// The language that the parser parses
    pub language: Format,
}

/// A trait for parsers
///
/// Defines similar functions to `serde_json` (and other `serde_` crates) for
/// converting nodes to/from strings, files, readers etc.
pub trait ParserTrait {
    /// Get the [`Parser`] specification
    fn spec() -> Parser;

    /// Parse some code into a [`ParseInfo`] object
    ///
    /// # Arguments
    ///
    /// - `code`: The code to parse
    /// - `path`: The filesystem path of the code. Used to resolve relative
    ///           file paths in parsed code.
    fn parse(code: &str, path: Option<&Path>) -> Result<ParseInfo>;
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize)]
#[serde(default, rename_all = "camelCase", crate = "common::serde")]
pub struct ParseInfo {
    /// The language that the code was parsed as
    pub language: Format,

    /// Whether the code has any syntax errors
    pub syntax_errors: bool,

    /// The semantic digest of the code
    pub semantic_digest: u64,

    /// The execution dependencies of the code
    pub execution_dependencies: Vec<ExecutionDependency>,

    /// The execution dependents of the code
    pub execution_dependents: Vec<ExecutionDependent>,

    /// The execution tags that were parsed
    pub execution_tags: Vec<ExecutionTag>,

    /// Whether the code has been tagged as pure or not
    pub execution_pure: Option<bool>,

    /// Whether the code has been tagged with an autorun option
    pub execution_auto: Option<ExecutionAuto>,
}

impl ParseInfo {
    /// Get the language the parse info is for
    pub fn language(&self) -> Format {
        self.language
    }

    /// Is the code pure (i.e. has no side effects)?
    ///
    /// If the code has not been explicitly tagged as pure or impure then
    /// returns `true` if there are no execution dependents.
    pub fn is_pure(&self) -> bool {
        self.execution_pure
            .unwrap_or(self.execution_dependents.is_empty())
    }

    /// Get a list of symbols (name and kind tuples) used by the code
    pub fn variables_used(&self) -> Vec<(String, Option<String>)> {
        self.execution_dependencies
            .iter()
            .filter_map(|dependency| {
                if let ExecutionDependencyNode::Variable(Variable { name, kind, .. }) =
                    &dependency.dependency_node
                {
                    Some((name.to_string(), kind.as_ref().map(|kind| kind.to_string())))
                } else {
                    None
                }
            })
            .collect_vec()
    }

    /// Get a list of symbols modified (name and kind tuples) by the code
    pub fn variables_modified(&self) -> Vec<(String, Option<String>)> {
        self.execution_dependents
            .iter()
            .filter_map(|dependent| {
                if let ExecutionDependentNode::Variable(Variable { name, kind, .. }) =
                    &dependent.dependent_node
                {
                    Some((name.to_string(), kind.as_ref().map(|kind| kind.to_string())))
                } else {
                    None
                }
            })
            .collect_vec()
    }

    /// Get the execution tags as a TagMap
    pub fn tag_map(&self) -> TagMap {
        // TODO: Avoid this clone
        TagMap::new(self.execution_tags.clone())
    }
}
