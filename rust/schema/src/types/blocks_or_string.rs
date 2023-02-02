//! Generated file, do not edit

use crate::prelude::*;

use super::blocks::Blocks;
use super::string::String;

/// [`Blocks`] or [`String`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]

pub enum BlocksOrString {
    Blocks(Blocks),
    String(String),
}
