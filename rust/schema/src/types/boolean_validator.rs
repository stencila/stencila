//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A schema specifying that a node must be a boolean value.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct BooleanValidator {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("BooleanValidator"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,
}
