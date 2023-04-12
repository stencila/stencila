// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::cite_or_string::CiteOrString;
use super::inline::Inline;
use super::string::String;

/// Inline, quoted content.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write)]
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
    #[rustfmt::skip]
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
