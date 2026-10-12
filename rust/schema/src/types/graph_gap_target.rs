// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::graph_edge_kind::GraphEdgeKind;
use super::graph_gap_direction::GraphGapDirection;
use super::graph_gap_target_kind::GraphGapTargetKind;
use super::string::String;

/// The structured scope of a graph limitation.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("GraphGapTarget")]
pub struct GraphGapTarget {
    /// The type of this item.
    pub r#type: MustBe!("GraphGapTarget"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "qmd", format = "tiptap")]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// Whether the limitation qualifies an item, property or relationship.
    pub kind: GraphGapTargetKind,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<GraphGapTargetOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct GraphGapTargetOptions {
    /// A property path, required for a Property target.
    pub property: Option<String>,

    /// The edge kind, required for a Relationship target.
    pub relationship: Option<GraphEdgeKind>,

    /// The edge direction relative to the containing item, required for a Relationship target.
    pub direction: Option<GraphGapDirection>,

    /// An optional relationship role narrowing its scope.
    pub role: Option<String>,
}

impl GraphGapTarget {
    const NICK: [u8; 3] = *b"ggt";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::GraphGapTarget
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(kind: GraphGapTargetKind) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }
}
