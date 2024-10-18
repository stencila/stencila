// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of an instruction describing the operation to be performed.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum InstructionType {
    /// Create new document content, usually a single document node (e.g. `Paragraph` or `Table`), ignoring any existing content nested within the instruction. The instruction message will normally include the type of content to produce (e.g. "paragraph", "table", "list"). 
    #[default]
    #[serde(alias = "New")]
    #[strum(serialize = "New")]
    Create,

    /// Edit existing document nodes. Expected to return the same node types as existing nodes. 
    Edit,

    /// Fix an existing document node, usually a `CodeChunk`, `CodeInline`, or `MathBlock`. Expected to return the same node type without any `compilationErrors` or `executionErrors`. 
    Fix,

    /// Describe other document content. The instruction message should indicate the target for the description e.g. "describe figure 1", "describe next", "describe prev output" 
    Describe,
}
