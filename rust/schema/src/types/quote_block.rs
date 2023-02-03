//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::cite_or_string::CiteOrString;
use super::string::String;

/// A section quoted from somewhere else.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct QuoteBlock {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("QuoteBlock"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The source of the quote.
    pub cite: Option<CiteOrString>,

    /// The content of the quote.
    pub content: Vec<Block>,
}
