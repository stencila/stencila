use crate::prelude::*;

/// The relation between a node and its execution dependency.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Read, Write, ToHtml)]
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
