// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The relation between a node and its execution dependency.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionDependencyRelation {
    /// The node calls its dependency (usually another document or function)
    Calls,

    /// The node is derived from its dependency (e.g. a database table)
    Derives,

    /// The node imports its dependency (usually a software module)
    Imports,

    /// The node includes its dependency (usually another document)
    Includes,

    /// The node reads its dependency (usually a file)
    Reads,

    /// The node uses its dependency (usually a variable)
    #[default]
    Uses,
}
