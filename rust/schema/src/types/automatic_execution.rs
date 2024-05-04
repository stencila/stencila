// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum AutomaticExecution {
    /// Never automatically execute the document node. Only execute the when the user explicitly executes the specific node or all nodes in the containing document. 
    Never,

    /// Automatically execute the document node when it needs to be: if it is stale and is  upstream dependency of a node that has been executed, or is a downstream dependant of a node that has been executed. 
    #[default]
    Needed,

    /// Always execute the code when one of its dependants is executed, even if it is not stale (i.e. it, or its own dependencies, are unchanged since the last time it was executed). 
    Always,
}
