// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// [`String`] or [`Number`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum StringOrNumber {
    #[default]
    String(String),

    Number(Number),
}
