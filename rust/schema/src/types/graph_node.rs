// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::graph_evidence::GraphEvidence;
use super::graph_gap::GraphGap;
use super::node::Node;
use super::string::String;

/// A node in a graph.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("GraphNode")]
pub struct GraphNode {
    /// The type of this item.
    pub r#type: MustBe!("GraphNode"),

    /// The durable graph-local id used by graph edges to reference this graph node.
    pub id: String,

    /// The embedded Stencila node represented by this graph node.
    pub node: Box<Node>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<GraphNodeOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodeOptions {
    /// Sourced limitations; absence means no gaps were reported, not complete coverage.
    #[serde(alias = "gap")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub gaps: Option<Vec<GraphGap>>,

    /// Evidence of this node's events, including events supporting no relationship.
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub evidence: Option<Vec<GraphEvidence>>,
}

impl GraphNode {
    const NICK: [u8; 3] = *b"gnd";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::GraphNode
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(id: String, node: Box<Node>) -> Self {
        Self {
            id,
            node,
            ..Default::default()
        }
    }
}
