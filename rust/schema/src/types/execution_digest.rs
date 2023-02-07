//! Generated file, do not edit

use crate::prelude::*;

use super::number::Number;

/// A digest of the execution state of a node.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionDigest {
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

impl ExecutionDigest {
    pub fn new(state_digest: Number, semantic_digest: Number, dependencies_digest: Number, dependencies_stale: Number, dependencies_failed: Number) -> Self {
        Self{
            state_digest,
            semantic_digest,
            dependencies_digest,
            dependencies_stale,
            dependencies_failed,
            ..Default::default()
        }
    }
}

