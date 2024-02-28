// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A type to indicate a value or or other type in unknown.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Unknown")]
pub struct Unknown {
    /// The type of this item.
    pub r#type: MustBe!("Unknown"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl Unknown {
    const NICK: [u8; 3] = [117, 110, 107];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Unknown
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
