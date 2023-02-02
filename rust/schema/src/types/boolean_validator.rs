//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A schema specifying that a node must be a boolean value.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct BooleanValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("BooleanValidator"),

    /// The identifier for this item
    id: Option<String>,
}
