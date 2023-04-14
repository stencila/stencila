use crate::prelude::*;

use super::property_value::PropertyValue;
use super::string::String;

/// [`PropertyValue`] or [`String`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]

pub enum PropertyValueOrString {
    PropertyValue(PropertyValue),
    String(String),
}
