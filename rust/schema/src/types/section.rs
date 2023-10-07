// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::string::String;

/// A section of a document
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "section")]
#[jats(elem = "sec")]
#[markdown(special)]
pub struct Section {
    /// The type of this item
    pub r#type: MustBe!("Section"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content within the section
    #[strip(types)]
    pub content: Vec<Block>,
}

impl Section {
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
