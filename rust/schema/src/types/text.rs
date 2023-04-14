// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::text_value::TextValue;

/// Textual content
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Text {
    /// The type of this item
    pub r#type: MustBe!("Text"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The value of the text content
    pub value: TextValue,
}

impl Text {
    #[rustfmt::skip]
    pub fn new(value: TextValue) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
