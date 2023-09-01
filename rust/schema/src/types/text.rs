// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::text_value::TextValue;

/// Textual content
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "span")]
pub struct Text {
    /// The type of this item
    pub r#type: MustBe!("Text"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The value of the text content
    #[html(content)]
    pub value: TextValue,
}
impl Text {
    pub fn new(value: TextValue) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
