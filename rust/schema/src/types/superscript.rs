//! Generated file, do not edit

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// Superscripted content.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Superscript {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Superscript"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The content that is marked.
    pub content: Vec<Inline>,
}
