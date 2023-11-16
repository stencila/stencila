// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::time::Time;

/// A validator specifying the constraints on a time.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "TimeValidator")]
pub struct TimeValidator {
    /// The type of this item.
    pub r#type: MustBe!("TimeValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The inclusive lower limit for a time.
    pub minimum: Option<Time>,

    /// The inclusive upper limit for a time.
    pub maximum: Option<Time>,
}

impl TimeValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
