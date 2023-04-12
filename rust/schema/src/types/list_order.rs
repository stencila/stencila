use crate::prelude::*;

/// Indicates how a `List` is ordered.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Read, Write)]
#[serde(untagged, crate = "common::serde")]
#[def = "Unordered"]
pub enum ListOrder {
    Ascending,
    Descending,
    Unordered,
}
