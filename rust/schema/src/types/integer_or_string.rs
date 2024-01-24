// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;

/// [`Integer`] or [`String`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault)]
#[serde(untagged, crate = "common::serde")]
pub enum IntegerOrString {
    #[default]
    Integer(Integer),

    String(String),
}
