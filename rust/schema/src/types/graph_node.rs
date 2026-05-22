// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A node in a graph.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
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

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
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
