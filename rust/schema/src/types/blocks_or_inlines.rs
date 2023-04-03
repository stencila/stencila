//! Generated file, do not edit

use crate::prelude::*;

use super::blocks::Blocks;
use super::inlines::Inlines;

/// [`Blocks`] or [`Inlines`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]

pub enum BlocksOrInlines {
    Blocks(Blocks),
    Inlines(Inlines),
}
