// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Under which circumstances a node should be executed.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionMode {
    /// Execute on demand only. 
    Demand,

    /// Execute on demand and, if the node is stale, when the document or ancestor node is executed. 
    #[default]
    Need,

    /// Execute on demand and whenever the document or ancestor node is executed. Use this for nodes that you want to always be executed, even if they, or their upstream dependencies, are not stale. 
    Always,

    /// Execute on demand, and automatically if it is stale, including if is an upstream dependency of a node that is to be executed, or is a downstream dependant of a node that has been executed. 
    Auto,

    /// Do not execute the node. Requires that the node is unlocked first to be executed. 
    Lock,
}
