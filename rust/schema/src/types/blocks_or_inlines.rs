// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::blocks::Blocks;
use super::inlines::Inlines;

/// [`Blocks`] or [`Inlines`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(untagged, crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum BlocksOrInlines {
    Blocks(Blocks),

    Inlines(Inlines),
}
