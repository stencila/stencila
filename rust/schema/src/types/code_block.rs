// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A code block.
#[rustfmt::skip]
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CodeBlock {
    /// The type of this item
    pub r#type: MustBe!("CodeBlock"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The code.
    pub code: String,

    /// The programming language of the code.
    pub programming_language: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<CodeBlockOptions>,
}

#[rustfmt::skip]
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CodeBlockOptions {
    /// Media type, typically expressed using a MIME format, of the code.
    pub media_type: Option<String>,
}

impl CodeBlock {
    #[rustfmt::skip]
    pub fn new(code: String) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }
}
