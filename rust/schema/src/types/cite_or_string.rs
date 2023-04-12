use crate::prelude::*;

use super::cite::Cite;
use super::string::String;

/// [`Cite`] or [`String`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write)]
#[serde(untagged, crate = "common::serde")]

pub enum CiteOrString {
    Cite(Cite),
    String(String),
}
