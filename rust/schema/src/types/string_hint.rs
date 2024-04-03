// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;

/// A hint to the structure of an `String`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, MergeNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "StringHint")]
pub struct StringHint {
    /// The type of this item.
    pub r#type: MustBe!("StringHint"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The number of characters in the string.
    pub chars: Integer,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl StringHint {
    const NICK: [u8; 3] = [115, 116, 104];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::StringHint
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(chars: Integer) -> Self {
        Self {
            chars,
            ..Default::default()
        }
    }
}
