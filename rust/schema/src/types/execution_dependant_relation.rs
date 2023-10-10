// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The relation between a node and its execution dependant.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, SmartDefault, ReadNode, WriteNode)]
#[serde(crate = "common::serde")]
pub enum ExecutionDependantRelation {
    #[default]
    Assigns,

    Alters,

    Declares,

    Writes,
}
