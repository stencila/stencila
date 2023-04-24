use crate::prelude::*;

/// The relation between a node and its execution dependant.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
#[def = "Assigns"]
pub enum ExecutionDependantRelation {
    Assigns,
    Alters,
    Declares,
    Writes,
}
