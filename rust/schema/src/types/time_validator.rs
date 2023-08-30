// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::time::Time;

/// A validator specifying the constraints on a time.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TimeValidator {
    /// The type of this item
    pub r#type: MustBe!("TimeValidator"),

    /// The identifier for this item
    #[strip(id)]
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
