// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::integer::Integer;
use super::string::String;

/// A heading.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(special)]
#[jats(elem = "title")]
#[markdown(special)]
pub struct Heading {
    /// The type of this item
    pub r#type: MustBe!("Heading"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The level of the heading.
    #[default = 0]
    pub level: Integer,

    /// Content of the heading.
    #[strip(types)]
    pub content: Vec<Inline>,
}

impl Heading {
    pub fn new(level: Integer, content: Vec<Inline>) -> Self {
        Self {
            level,
            content,
            ..Default::default()
        }
    }
}
