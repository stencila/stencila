// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// A digest of the execution state of a node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ExecutionDigest")]
pub struct ExecutionDigest {
    /// The type of this item.
    pub r#type: MustBe!("ExecutionDigest"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A digest of the state of a node.
    #[serde(alias = "state-digest", alias = "state_digest")]
    pub state_digest: Number,

    /// A digest of the "semantic intent" of the resource with respect to the dependency graph
    #[serde(alias = "semantic-digest", alias = "semantic_digest")]
    pub semantic_digest: Number,

    /// A digest of the semantic digests the dependencies of a resource.
    #[serde(alias = "dependencies-digest", alias = "dependencies_digest")]
    pub dependencies_digest: Number,

    /// A count of the number of execution dependencies that are stale
    #[serde(alias = "dependencies-stale", alias = "dependencies_stale")]
    pub dependencies_stale: Number,

    /// A count of the number of execution dependencies that failed
    #[serde(alias = "dependencies-failed", alias = "dependencies_failed")]
    pub dependencies_failed: Number,
}

impl ExecutionDigest {}
