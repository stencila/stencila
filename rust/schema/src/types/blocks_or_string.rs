use crate::prelude::*;

use super::blocks::Blocks;
use super::string::String;

/// [`Blocks`] or [`String`]
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]

pub enum BlocksOrString {
    Blocks(Blocks),
    String(String),
}
