use crate::prelude::*;

use super::blocks::Blocks;
use super::inlines::Inlines;

/// [`Blocks`] or [`Inlines`]
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]

pub enum BlocksOrInlines {
    Blocks(Blocks),
    Inlines(Inlines),
}
