// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::date_time::DateTime;
use super::string::String;

/// A validator specifying the constraints on a date-time.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
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
    pub minimum: Option<DateTime>,

    /// The inclusive upper limit for a date-time.
    pub maximum: Option<DateTime>,

    /// A unique identifier for this node
    
    #[serde(skip)]
    pub node_id: NodeId
}

impl DateTimeValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Entity for DateTimeValidator {
    fn node_type() -> NodeType {
        NodeType::DateTimeValidator
    }

    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}
