// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::integer::Integer;
use super::string::String;

/// A heading.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Heading {
    /// The type of this item
    pub r#type: MustBe!("Heading"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The depth of the heading.
    #[default = 1]
    pub depth: Integer,

    /// Content of the heading.
    pub content: Vec<Inline>,
}
impl Heading {
    pub fn new(depth: Integer, content: Vec<Inline>) -> Self {
        Self {
            depth,
            content,
            ..Default::default()
        }
    }
}
