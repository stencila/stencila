// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;

/// [`Integer`] or [`String`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, HtmlCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum IntegerOrString {
    Integer(Integer),
    String(String),
}
