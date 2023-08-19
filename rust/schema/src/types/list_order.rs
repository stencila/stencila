// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates how a `List` is ordered.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
pub enum ListOrder {
    Ascending,
    Descending,
    #[default]
    Unordered,
}
