// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::property_value::PropertyValue;
use super::string::String;

/// [`PropertyValue`] or [`String`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml, ToText)]
#[serde(untagged, crate = "common::serde")]
pub enum PropertyValueOrString {
    PropertyValue(PropertyValue),
    String(String),
}
