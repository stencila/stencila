use crate::prelude::*;

use super::number::Number;
use super::string::String;

/// [`String`] or [`Number`]
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]

pub enum StringOrNumber {
    String(String),
    Number(Number),
}
