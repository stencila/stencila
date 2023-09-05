// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "hr")]
#[markdown(format = "---\n\n")]
pub struct ThematicBreak {
    /// The type of this item
    pub r#type: MustBe!("ThematicBreak"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,
}
impl ThematicBreak {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
