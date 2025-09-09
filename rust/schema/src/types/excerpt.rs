// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::reference::Reference;
use super::string::String;

/// An excerpt from a `CreativeWork`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("Excerpt")]
pub struct Excerpt {
    /// The type of this item.
    pub r#type: MustBe!("Excerpt"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// A `Reference` to the `CreativeWork` that the excerpt was taken from.
    #[strip(metadata)]
    #[dom(elem = "div")]
    pub source: Reference,

    /// The path to the node that was excepted.
    #[serde(alias = "node-path", alias = "node_path")]
    #[strip(metadata)]
    pub node_path: String,

    /// The types of the ancestor nodes and the node that was excerpted.
    #[serde(alias = "node-types", alias = "node_types")]
    #[strip(metadata)]
    pub node_types: String,

    /// The excerpted content.
    #[serde(deserialize_with = "one_or_many")]
    #[strip(content)]
    #[walk]
    #[patch(format = "all")]
    #[dom(elem = "div")]
    pub content: Vec<Block>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl Excerpt {
    const NICK: [u8; 3] = *b"exc";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Excerpt
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(source: Reference, node_path: String, node_types: String, content: Vec<Block>) -> Self {
        Self {
            source,
            node_path,
            node_types,
            content,
            ..Default::default()
        }
    }
}
