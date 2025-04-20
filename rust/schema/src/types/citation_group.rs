// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::citation::Citation;
use super::string::String;

/// A group of `Citation` nodes.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("CitationGroup")]
#[jats(special)]
pub struct CitationGroup {
    /// The type of this item.
    pub r#type: MustBe!("CitationGroup"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// One or more `Citation`s to be referenced in the same surrounding text.
    #[serde(alias = "item")]
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[dom(elem = "span")]
    pub items: Vec<Citation>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl CitationGroup {
    const NICK: [u8; 3] = [99, 116, 103];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::CitationGroup
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(items: Vec<Citation>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }
}
