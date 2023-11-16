// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::postal_address::PostalAddress;
use super::string::String;

/// [`PostalAddress`] or [`String`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(untagged, crate = "common::serde")]
pub enum PostalAddressOrString {
    PostalAddress(PostalAddress),

    String(String),
}
