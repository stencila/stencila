// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// [`UnsignedInteger`] or [`String`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum UnsignedIntegerOrString {
    #[default]
    UnsignedInteger(UnsignedInteger),

    String(String),
}
