// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::chat_message::ChatMessage;
use super::string::String;

/// A group of messages, usually alternative model messages, within a `Chat`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ChatMessageGroup")]
#[patch(apply_with = "ChatMessageGroup::apply_patch_op")]
pub struct ChatMessageGroup {
    /// The type of this item.
    pub r#type: MustBe!("ChatMessageGroup"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The messages within the group.
    #[serde(alias = "message")]
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[dom(elem = "div")]
    pub messages: Vec<ChatMessage>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ChatMessageGroup {
    const NICK: [u8; 3] = [99, 109, 103];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ChatMessageGroup
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        Self {
            messages,
            ..Default::default()
        }
    }
}
