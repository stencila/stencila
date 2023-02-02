//! Generated file, do not edit

use crate::prelude::*;

use super::number::Number;

/// A digest of the execution state of a node.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct ExecutionDigest {
    /// A digest of the state of a node.
    state_digest: Number,

    /// A digest of the "semantic intent" of the resource with respect to the dependency graph
    semantic_digest: Number,

    /// A digest of the semantic digests the dependencies of a resource.
    dependencies_digest: Number,

    /// A count of the number of execution dependencies that are stale
    dependencies_stale: Number,

    /// A count of the number of execution dependencies that failed
    dependencies_failed: Number,
}
