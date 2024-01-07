// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::message_part::MessagePart;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::string::String;

/// A message from a sender to one or more people, organizations or software application.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
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
    pub parts: Vec<MessagePart>,

    /// The sender of the message.
    pub sender: Option<PersonOrOrganizationOrSoftwareApplication>,
}

impl Message {
    pub fn new(parts: Vec<MessagePart>) -> Self {
        Self {
            parts,
            ..Default::default()
        }
    }
}
