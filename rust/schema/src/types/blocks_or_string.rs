use crate::prelude::*;

use super::blocks::Blocks;
use super::string::String;

/// [`Blocks`] or [`String`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write)]
#[serde(untagged, crate = "common::serde")]

pub enum BlocksOrString {
    Blocks(Blocks),
    String(String),
}
