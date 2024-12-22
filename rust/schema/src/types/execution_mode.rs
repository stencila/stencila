// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Under which circumstances a node should be executed.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionMode {
    /// Use the mode configured in the document, workspace or user settings. 
    #[default]
    Default,

    /// Execute the node when needed (i.e. if it is stale) when the document or ancestor node is executed. 
    Need,

    /// Always execute the node when the document or ancestor node is executed. Use this for nodes that you want to always be executed, even if they, or their upstream dependencies, are not stale. 
    Always,

    /// Automatically execute the node if it is stale and is an upstream dependency of a node that is to be executed, or is a downstream dependant of a node that has been executed. 
    Auto,

    /// Do not execute the node. Requires that the node is unlocked first to be executed. 
    Lock,
}
