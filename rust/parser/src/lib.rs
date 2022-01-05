use eyre::Result;
use graph_triples::{resources::Symbol, Pairs, Relation, Resource, ResourceId};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{collections::BTreeMap, path::Path};

// Export and re-export for the convenience of crates that implement a parser
pub mod utils;
pub use eyre;
pub use formats;
pub use graph_triples;

/// A specification for parsers
///
/// All parsers, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Parser` instance from the
/// `spec` function of `ParserTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Parser {
    /// The language that the parser parses
    pub language: String,
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

    /// Parse some code and return a set of graph pairs
    fn parse(path: &Path, code: &str) -> Result<ParseInfo>;
}

/// The result of parsing
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct ParseInfo {
    /// Whether the code had an explicit `@pure` or `@impure` tag
    pub pure: Option<bool>,

    /// The [`Relation`]-[`Resource`] pairs between the code and other resources
    /// (e.g. `Symbol`s, `File`s)
    pub relations: Pairs,
}

impl ParseInfo {
    /// Is the parsed code pure (i.e. has no side effects)?
    ///
    /// If the code has not been explicitly tagged as `@pure` or `@impure` then
    /// returns `true` if there are any side-effect causing relations.
    pub fn is_pure(&self) -> bool {
        self.pure.unwrap_or_else(|| {
            self.relations
                .iter()
                .filter(|(relation, ..)| {
                    matches!(
                        relation,
                        Relation::Assign(..)
                            | Relation::Alter(..)
                            | Relation::Import(..)
                            | Relation::Write(..)
                    )
                })
                .count()
                == 0
        })
    }

    /// Get a list of symbols used by the parsed code
    pub fn symbols_used(&self) -> Vec<Symbol> {
        self.relations
            .iter()
            .filter_map(|pair| match pair {
                (Relation::Use(..), Resource::Symbol(symbol)) => Some(symbol),
                _ => None,
            })
            .cloned()
            .collect()
    }

    /// Get a list of symbols modified by the code
    pub fn symbols_modified(&self) -> Vec<Symbol> {
        self.relations
            .iter()
            .filter_map(|pair| match pair {
                (Relation::Assign(..), Resource::Symbol(symbol))
                | (Relation::Alter(..), Resource::Symbol(symbol)) => Some(symbol),
                _ => None,
            })
            .cloned()
            .collect()
    }
}

/// A map of node ids to their `ParseInfo`
///
/// A `BTreeMap` is used instead of a `HashMap` for determinism in order
/// of entries.
pub type ParseMap = BTreeMap<ResourceId, ParseInfo>;
