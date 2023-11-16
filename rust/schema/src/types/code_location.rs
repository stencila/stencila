// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// The location within some source code.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "CodeLocation")]
pub struct CodeLocation {
    /// The type of this item.
    pub r#type: MustBe!("CodeLocation"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The source of the code, a file path, label or URL.
    pub source: Option<String>,

    /// The 1-based index if the first line on which the error occurred.
    #[serde(alias = "start-line", alias = "start_line")]
    pub start_line: Option<UnsignedInteger>,

    /// The 1-based index if the first column on which the error occurred.
    #[serde(alias = "start-column", alias = "start_column")]
    pub start_column: Option<UnsignedInteger>,

    /// The 1-based index if the last line on which the error occurred.
    #[serde(alias = "end-line", alias = "end_line")]
    pub end_line: Option<UnsignedInteger>,

    /// The 1-based index if the last column on which the error occurred.
    #[serde(alias = "end-column", alias = "end_column")]
    pub end_column: Option<UnsignedInteger>,
}

impl CodeLocation {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
