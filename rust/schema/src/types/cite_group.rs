// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cite::Cite;
use super::string::String;

/// A group of `Cite` nodes.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "CiteGroup")]
pub struct CiteGroup {
    /// The type of this item.
    pub r#type: MustBe!("CiteGroup"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// One or more `Cite`s to be referenced in the same surrounding text.
    #[serde(alias = "item")]
    #[serde(deserialize_with = "one_or_many")]
    pub items: Vec<Cite>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl CiteGroup {
    const NICK: &'static str = "ctg";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::CiteGroup
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(items: Vec<Cite>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }
}
