use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// [`String`] or [`Number`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write)]
#[serde(untagged, crate = "common::serde")]

pub enum StringOrNumber {
    String(String),
    Number(Number),
}
