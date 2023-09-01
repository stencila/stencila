// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates how a `List` is ordered.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, SmartDefault, Read, Write)]
#[serde(crate = "common::serde")]
pub enum ListOrder {
    Ascending,
    Descending,
    #[default]
    Unordered,
}
