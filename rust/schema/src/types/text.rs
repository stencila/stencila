//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// Textual content
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Text {
    /// The type of this item
    pub r#type: MustBe!("Text"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The value of the text content
    pub value: String,
}

impl Text {
    #[rustfmt::skip]
    pub fn new(value: String) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
