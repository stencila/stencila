// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A positional marker within inline content used to define the boundary of a cross-block range. Boundaries are referenced by their `id` from other nodes (e.g. `Comment.startLocation` and `Comment.endLocation`) to delimit regions that may span across multiple blocks.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("Boundary")]
pub struct Boundary {
    /// The type of this item.
    pub r#type: MustBe!("Boundary"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl Boundary {
    const NICK: [u8; 3] = *b"bdy";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Boundary
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
