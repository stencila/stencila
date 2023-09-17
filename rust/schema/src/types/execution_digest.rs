// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// A digest of the execution state of a node.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionDigest {
    /// The type of this item
    pub r#type: MustBe!("ExecutionDigest"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A digest of the state of a node.
    pub state_digest: Number,

    /// A digest of the "semantic intent" of the resource with respect to the dependency graph
    pub semantic_digest: Number,

    /// A digest of the semantic digests the dependencies of a resource.
    pub dependencies_digest: Number,

    /// A count of the number of execution dependencies that are stale
    pub dependencies_stale: Number,

    /// A count of the number of execution dependencies that failed
    pub dependencies_failed: Number,
}

impl ExecutionDigest {}
