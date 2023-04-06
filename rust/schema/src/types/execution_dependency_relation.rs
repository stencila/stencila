use crate::prelude::*;

/// The relation between a node and its execution dependency.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
#[def = "Uses"]
pub enum ExecutionDependencyRelation {
    Calls,
    Derives,
    Imports,
    Includes,
    Reads,
    Uses,
}
