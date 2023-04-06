use crate::prelude::*;

/// The relation between a node and its execution dependant.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
#[def = "Assigns"]
pub enum ExecutionDependantRelation {
    Assigns,
    Alters,
    Declares,
    Writes,
}
