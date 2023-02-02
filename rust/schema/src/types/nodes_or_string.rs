//! Generated file, do not edit

use crate::prelude::*;

use super::nodes::Nodes;
use super::string::String;

/// [`Nodes`] or [`String`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]

pub enum NodesOrString {
    Nodes(Nodes),
    String(String),
}
