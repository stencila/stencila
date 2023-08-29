// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cite_or_string::CiteOrString;
use super::inline::Inline;
use super::string::String;

/// Inline, quoted content.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml, ToText)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Quote {
    /// The type of this item
    pub r#type: MustBe!("Quote"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The content that is marked.
    pub content: Vec<Inline>,

    /// The source of the quote.
    pub cite: Option<CiteOrString>,
}
impl Quote {
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
