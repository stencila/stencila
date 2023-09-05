// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::blocks::Blocks;
use super::string::String;

/// [`Blocks`] or [`String`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, Read, Write)]
#[serde(untagged, crate = "common::serde")]
pub enum BlocksOrString {
    Blocks(Blocks),
    String(String),
}
