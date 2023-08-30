// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::blocks::Blocks;
use super::inlines::Inlines;

/// [`Blocks`] or [`Inlines`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, Read, Write)]
#[serde(untagged, crate = "common::serde")]
pub enum BlocksOrInlines {
    Blocks(Blocks),
    Inlines(Inlines),
}
