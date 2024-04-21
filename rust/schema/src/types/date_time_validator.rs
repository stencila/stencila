// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::date_time::DateTime;
use super::string::String;

/// A validator specifying the constraints on a date-time.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "DateTimeValidator")]
pub struct DateTimeValidator {
    /// The type of this item.
    pub r#type: MustBe!("DateTimeValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The inclusive lower limit for a date-time.
    #[patch(format = "md")]
    pub minimum: Option<DateTime>,

    /// The inclusive upper limit for a date-time.
    #[patch(format = "md")]
    pub maximum: Option<DateTime>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl DateTimeValidator {
    const NICK: [u8; 3] = [100, 116, 118];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::DateTimeValidator
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
