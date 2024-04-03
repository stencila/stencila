// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;

/// [`Integer`] or [`String`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, MergeNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum IntegerOrString {
    #[default]
    Integer(Integer),

    String(String),
}
