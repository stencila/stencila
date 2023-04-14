// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::cite_or_string::CiteOrString;
use super::string::String;

/// A section quoted from somewhere else.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
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
    #[rustfmt::skip]
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
