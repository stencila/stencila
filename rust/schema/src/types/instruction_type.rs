// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of an instruction describing the operation to be performed.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum InstructionType {
    /// Discuss document, kernel, workspace or other contexts. Normally only used for `Chat`s. 
    Discuss,

    /// Create new document content, usually a single document node (e.g. `Paragraph` or `Table`), ignoring any existing content nested within the instruction. The instruction message will normally include the type of content to produce (e.g. "paragraph", "table", "list"). 
    #[default]
    #[serde(alias = "New")]
    Create,

    /// Describe other document content. The instruction message should indicate the target for the description e.g. "describe figure 1", "describe next", "describe prev output" 
    Describe,

    /// Edit existing document nodes. Expected to return the same node types as existing nodes. 
    Edit,

    /// Fix an existing document node, usually a `CodeChunk`, `CodeInline`, or `MathBlock`. Expected to return the same node type without any `compilationErrors` or `executionErrors`. 
    Fix,
}
