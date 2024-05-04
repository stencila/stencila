// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The relation between a node and its execution dependant.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionDependantRelation {
    /// The node assigns its dependant (usually a variable)
    #[default]
    Assigns,

    /// The node alters its dependant (usually a variable)
    Alters,

    /// The node declares its dependant (e.g. a database table)
    Declares,

    /// The node writes its dependant (usually a file)
    Writes,
}
