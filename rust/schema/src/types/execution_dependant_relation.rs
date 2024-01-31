// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The relation between a node and its execution dependant.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionDependantRelation {
    #[default]
    Assigns,

    Alters,

    Declares,

    Writes,
}
