use std::path::Path;

use common::{
    eyre::Result,
    serde::{Deserialize, Serialize},
};
use formats::Format;
use graph_triples::{Resource, ResourceInfo};

// Export and re-export for the convenience of crates that implement a parser
pub mod utils;
pub use common;
pub use formats;
pub use graph_triples;

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
/// This trait can be used by Rust implementations of parsers, allowing them to
/// be compiled into the Stencila binaries.
///
/// It defines similar functions to `serde_json` (and other `serde_` crates) for
/// converting nodes to/from strings, files, readers etc.
pub trait ParserTrait {
    /// Get the [`Parser`] specification
    fn spec() -> Parser;

    /// Parse a [`Resource::Code`] object into a [`ResourceInfo`] object
    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo>;
}
