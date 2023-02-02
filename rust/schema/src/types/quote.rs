//! Generated file, do not edit

use crate::prelude::*;

use super::cite_or_string::CiteOrString;
use super::inline::Inline;
use super::string::String;

/// Inline, quoted content.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct Quote {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("Quote"),

    /// The identifier for this item
    id: Option<String>,

    /// The content that is marked.
    content: Vec<Inline>,

    /// The source of the quote.
    cite: Option<CiteOrString>,
}
