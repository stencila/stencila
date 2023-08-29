// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::blocks::Blocks;
use super::string::String;

/// [`Blocks`] or [`String`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, HtmlCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum BlocksOrString {
    Blocks(Blocks),
    String(String),
}
