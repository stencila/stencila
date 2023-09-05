// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::postal_address::PostalAddress;
use super::string::String;

/// [`PostalAddress`] or [`String`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, Read, Write)]
#[serde(untagged, crate = "common::serde")]
pub enum PostalAddressOrString {
    PostalAddress(PostalAddress),
    String(String),
}
