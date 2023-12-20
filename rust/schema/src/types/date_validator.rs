// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::date::Date;
use super::string::String;

/// A validator specifying the constraints on a date.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "DateValidator")]
pub struct DateValidator {
    /// The type of this item.
    pub r#type: MustBe!("DateValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The inclusive lower limit for a date.
    pub minimum: Option<Date>,

    /// The inclusive upper limit for a date.
    pub maximum: Option<Date>,

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl DateValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Entity for DateValidator {
    const NICK: &'static str = "dat";

    fn node_type(&self) -> NodeType {
        NodeType::DateValidator
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
