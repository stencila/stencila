// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The relation between a node and its execution dependency.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, SmartDefault, Read, Write)]
#[serde(crate = "common::serde")]
pub enum ExecutionDependencyRelation {
    Calls,
    Derives,
    Imports,
    Includes,
    Reads,
    #[default]
    Uses,
}
