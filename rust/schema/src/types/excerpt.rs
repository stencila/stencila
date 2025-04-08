// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::reference::Reference;
use super::string::String;

/// An excerpt from a `CreativeWork`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("Excerpt")]
pub struct Excerpt {
    /// The type of this item.
    pub r#type: MustBe!("Excerpt"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A `Reference` to the `CreativeWork` that the excerpt was taken from.
    #[strip(metadata)]
    #[dom(elem = "div")]
    pub source: Reference,

    /// A `Reference` to the `CreativeWork` that the excerpt was taken from.
    #[serde(alias = "node-path", alias = "node_path")]
    #[strip(metadata)]
    pub node_path: String,

    /// The route to the node that was excerpted including the .
    #[serde(alias = "node-ancestors", alias = "node_ancestors")]
    #[strip(metadata)]
    pub node_ancestors: String,

    /// The type of the node that was excerpted.
    #[serde(alias = "node-type", alias = "node_type")]
    #[strip(metadata)]
    pub node_type: String,

    /// The excerpted content.
    #[serde(deserialize_with = "one_or_many")]
    #[strip(content)]
    #[patch(format = "all")]
    #[dom(elem = "div")]
    pub content: Vec<Block>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl Excerpt {
    const NICK: [u8; 3] = [101, 120, 99];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Excerpt
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
}
