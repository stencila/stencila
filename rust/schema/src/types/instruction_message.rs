// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::message_level::MessageLevel;
use super::message_part::MessagePart;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::string::String;

/// A message within an `Instruction`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "InstructionMessage")]
pub struct InstructionMessage {
    /// The type of this item.
    pub r#type: MustBe!("InstructionMessage"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Parts of the message.
    #[serde(alias = "part")]
    #[serde(deserialize_with = "one_or_many")]
    #[merge(format = "md")]
    #[dom(elem = "div")]
    pub parts: Vec<MessagePart>,

    /// Content of the message.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[merge(format = "all")]
    #[dom(elem = "div")]
    pub content: Option<Vec<Block>>,

    /// The authors of the message.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[dom(elem = "div")]
    pub authors: Option<Vec<PersonOrOrganizationOrSoftwareApplication>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<InstructionMessageOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct InstructionMessageOptions {
    /// The severity level of the message.
    pub level: Option<MessageLevel>,
}

impl InstructionMessage {
    const NICK: [u8; 3] = [105, 109, 101];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::InstructionMessage
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(parts: Vec<MessagePart>) -> Self {
        Self {
            parts,
            ..Default::default()
        }
    }
}
