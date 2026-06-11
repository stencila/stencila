// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::research_object_relation_kind::ResearchObjectRelationKind;
use super::string::String;

/// A relation from one research object to another.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("ResearchObjectRelation")]
pub struct ResearchObjectRelation {
    /// The type of this item.
    pub r#type: MustBe!("ResearchObjectRelation"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "qmd", format = "tiptap")]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The kind of relation.
    pub kind: ResearchObjectRelationKind,

    /// The target research object or external resource.
    pub target: String,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl ResearchObjectRelation {
    const NICK: [u8; 3] = *b"ror";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ResearchObjectRelation
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(kind: ResearchObjectRelationKind, target: String) -> Self {
        Self {
            kind,
            target,
            ..Default::default()
        }
    }
}
