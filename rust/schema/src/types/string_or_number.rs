// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// [`String`] or [`Number`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml, ToText)]
#[serde(untagged, crate = "common::serde")]
pub enum StringOrNumber {
    String(String),
    Number(Number),
}
