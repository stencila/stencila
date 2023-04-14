// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// An error that occurred when parsing, compiling or executing a Code node.
#[rustfmt::skip]
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CodeError {
    /// The type of this item
    pub r#type: MustBe!("CodeError"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The error message or brief description of the error.
    pub error_message: String,

    /// The type of error e.g. "SyntaxError", "ZeroDivisionError".
    pub error_type: Option<String>,

    /// Stack trace leading up to the error.
    pub stack_trace: Option<String>,
}

impl CodeError {
    #[rustfmt::skip]
    pub fn new(error_message: String) -> Self {
        Self {
            error_message,
            ..Default::default()
        }
    }
}
