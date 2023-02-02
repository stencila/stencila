//! Generated file, do not edit

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;

/// A schema specifying constraints on a string node.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct StringValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("StringValidator"),

    /// The identifier for this item
    id: String,

    /// The minimum length for a string node.
    min_length: Option<Integer>,

    /// The maximum length for a string node.
    max_length: Option<Integer>,

    /// A regular expression that a string node must match.
    pattern: Option<String>,
}
