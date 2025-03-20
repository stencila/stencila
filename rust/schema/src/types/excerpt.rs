// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::creative_work_type::CreativeWorkType;
use super::string::String;

/// An excerpt from a `CreativeWork`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
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

    /// The `CreativeWork` that the except was taken from.
    #[strip(metadata)]
    pub source: Box<CreativeWorkType>,

    /// The excerpted content.
    #[serde(deserialize_with = "one_or_many")]
    #[strip(content)]
    #[walk]
    #[patch(format = "all")]
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
    
    pub fn new(source: Box<CreativeWorkType>, content: Vec<Block>) -> Self {
        Self {
            source,
            content,
            ..Default::default()
        }
    }
}
