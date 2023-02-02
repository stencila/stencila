//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::cite_or_string::CiteOrString;
use super::string::String;

/// A section quoted from somewhere else.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct QuoteBlock {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("QuoteBlock"),

    /// The identifier for this item
    id: Option<String>,

    /// The source of the quote.
    cite: Option<CiteOrString>,

    /// The content of the quote.
    content: Vec<Block>,
}
