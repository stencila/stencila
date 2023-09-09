// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cord::Cord;
use super::string::String;

/// A code block.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "pre", custom)]
#[markdown(special)]
pub struct CodeBlock {
    /// The type of this item
    pub r#type: MustBe!("CodeBlock"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code.
    #[html(content)]
    pub code: Cord,

    /// The programming language of the code.
    pub programming_language: Option<String>,
}

impl CodeBlock {
    pub fn new(code: Cord) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }
}
