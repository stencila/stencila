// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// A suggestion to insert some inline content.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "ins", custom)]
#[jats(special)]
#[markdown(format = "<ins>{content}</ins>")]
pub struct Insert {
    /// The type of this item
    pub r#type: MustBe!("Insert"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content that is suggested to be inserted or deleted.
    pub content: Vec<Inline>,
}

impl Insert {
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
