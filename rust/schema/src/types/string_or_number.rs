// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// [`String`] or [`Number`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode)]
#[serde(untagged, crate = "common::serde")]
pub enum StringOrNumber {
    String(String),

    Number(Number),
}
