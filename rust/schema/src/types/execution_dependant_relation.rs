use crate::prelude::*;

/// The relation between a node and its execution dependant.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Defaults, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
#[def = "Assigns"]
pub enum ExecutionDependantRelation {
    Assigns,
    Alters,
    Declares,
    Writes,
}
