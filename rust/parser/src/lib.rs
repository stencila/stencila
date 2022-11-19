use std::path::Path;

use common::{
    eyre::Result,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
};
use formats::Format;
use stencila_schema::{ExecutionAuto, ExecutionDependency, ExecutionDependent, ExecutionTag};

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
#[derive(Debug, Default, Serialize)]
#[serde(default, rename_all = "camelCase", crate = "common::serde")]
pub struct ParseInfo {
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
