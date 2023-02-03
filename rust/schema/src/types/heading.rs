//! Generated file, do not edit

use crate::prelude::*;

use super::inline::Inline;
use super::integer::Integer;
use super::string::String;

/// A heading.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Heading {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Heading"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The depth of the heading.
    #[def = "1"]
    pub depth: Integer,

    /// Content of the heading.
    pub content: Vec<Inline>,
}
