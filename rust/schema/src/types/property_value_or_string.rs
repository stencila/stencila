// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::property_value::PropertyValue;
use super::string::String;

/// [`PropertyValue`] or [`String`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode)]
#[serde(untagged, crate = "common::serde")]
pub enum PropertyValueOrString {
    PropertyValue(PropertyValue),

    String(String),
}
