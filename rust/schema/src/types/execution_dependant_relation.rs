// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The relation between a node and its execution dependant.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
pub enum ExecutionDependantRelation {
    #[default]
    Assigns,
    Alters,
    Declares,
    Writes,
}
