//! Generated file, do not edit

use crate::prelude::*;

/// Indicates how a `List` is ordered.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(untagged, crate = "common::serde")]
#[def = "Unordered"]
pub enum ListOrder {
    Ascending,
    Descending,
    Unordered,
}
