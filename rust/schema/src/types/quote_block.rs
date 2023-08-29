// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::cite_or_string::CiteOrString;
use super::string::String;

/// A section quoted from somewhere else.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml, ToText)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct QuoteBlock {
    /// The type of this item
    pub r#type: MustBe!("QuoteBlock"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The source of the quote.
    pub cite: Option<CiteOrString>,

    /// The content of the quote.
    pub content: Vec<Block>,
}
impl QuoteBlock {
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
