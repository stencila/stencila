// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// An error that occurred when parsing, compiling or executing a `Code` node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "CodeError")]
pub struct CodeError {
    /// The type of this item.
    pub r#type: MustBe!("CodeError"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The error message or brief description of the error.
    #[serde(alias = "message", alias = "error-message", alias = "error_message")]
    pub error_message: String,

    /// The type of error e.g. "SyntaxError", "ZeroDivisionError".
    #[serde(alias = "error-type", alias = "error_type")]
    pub error_type: Option<String>,

    /// Stack trace leading up to the error.
    #[serde(alias = "trace", alias = "stack-trace", alias = "stack_trace")]
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
