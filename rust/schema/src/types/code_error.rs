// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// An error that occurred when parsing, compiling or executing a Code node.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CodeError {
    /// The type of this item
    pub r#type: MustBe!("CodeError"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The error message or brief description of the error.
    pub error_message: String,

    /// The type of error e.g. "SyntaxError", "ZeroDivisionError".
    pub error_type: Option<String>,

    /// Stack trace leading up to the error.
    pub stack_trace: Option<String>,
}

impl CodeError {
    pub fn new(error_message: String) -> Self {
        Self {
            error_message,
            ..Default::default()
        }
    }
}
