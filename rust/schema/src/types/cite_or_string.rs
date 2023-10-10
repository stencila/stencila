// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cite::Cite;
use super::string::String;

/// [`Cite`] or [`String`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(untagged, crate = "common::serde")]
pub enum CiteOrString {
    Cite(Cite),

    String(String),
}
