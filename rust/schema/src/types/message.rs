// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::message_part::MessagePart;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::string::String;

/// A message from a sender to one or more people, organizations or software application.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Message")]
pub struct Message {
    /// The type of this item.
    pub r#type: MustBe!("Message"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Parts of the message.
    #[serde(alias = "part")]
    #[serde(deserialize_with = "one_or_many")]
    #[dom(elem = "div")]
    pub parts: Vec<MessagePart>,

    /// Content of the message.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[dom(elem = "div")]
    pub content: Option<Vec<Block>>,

    /// The authors of the message.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[dom(elem = "div")]
    pub authors: Option<Vec<PersonOrOrganizationOrSoftwareApplication>>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl Message {
    const NICK: &'static str = "msg";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Message
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(parts: Vec<MessagePart>) -> Self {
        Self {
            parts,
            ..Default::default()
        }
    }
}
