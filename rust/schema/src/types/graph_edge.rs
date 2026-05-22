// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::graph_action::GraphAction;
use super::graph_edge_kind::GraphEdgeKind;
use super::graph_evidence::GraphEvidence;
use super::string::String;

/// A directed edge in a graph.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("GraphEdge")]
pub struct GraphEdge {
    /// The type of this item.
    pub r#type: MustBe!("GraphEdge"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The id of the upstream dependency graph node.
    pub source: String,

    /// The id of the downstream dependant graph node.
    pub target: String,

    /// The kind of dependency relationship represented by this edge.
    pub kind: GraphEdgeKind,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<GraphEdgeOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdgeOptions {
    /// Evidence supporting the edge.
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub evidence: Option<Vec<GraphEvidence>>,

    /// Concrete activities associated with the edge.
    #[serde(alias = "action")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub actions: Option<Vec<GraphAction>>,
}

impl GraphEdge {
    const NICK: [u8; 3] = *b"ged";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::GraphEdge
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(source: String, target: String, kind: GraphEdgeKind) -> Self {
        Self {
            source,
            target,
            kind,
            ..Default::default()
        }
    }
}
