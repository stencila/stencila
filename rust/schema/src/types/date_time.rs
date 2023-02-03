//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;

/// A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DateTime {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("DateTime"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The date as an ISO 8601 string.
    pub value: String,
}
