use crate::prelude::*;

/// Indicates how a `List` is ordered.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Defaults, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
#[def = "Unordered"]
pub enum ListOrder {
    Ascending,
    Descending,
    Unordered,
}
