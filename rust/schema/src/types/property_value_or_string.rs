//! Generated file, do not edit

use crate::prelude::*;

use super::property_value::PropertyValue;
use super::string::String;

/// [`PropertyValue`] or [`String`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(untagged, crate = "common::serde")]

pub enum PropertyValueOrString {
    PropertyValue(PropertyValue),
    String(String),
}
