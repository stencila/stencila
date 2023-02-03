//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A point in time recurring on multiple days
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Time {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Time"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`.
    pub value: String,
}
