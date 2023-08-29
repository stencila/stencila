// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::blocks::Blocks;
use super::inlines::Inlines;

/// [`Blocks`] or [`Inlines`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml, ToText)]
#[serde(untagged, crate = "common::serde")]
pub enum BlocksOrInlines {
    Blocks(Blocks),
    Inlines(Inlines),
}
