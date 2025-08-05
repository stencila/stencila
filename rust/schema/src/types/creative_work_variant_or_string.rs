// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::creative_work_variant::CreativeWorkVariant;
use super::string::String;

/// [`CreativeWorkVariant`] or [`String`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum CreativeWorkVariantOrString {
    #[default]
    CreativeWorkVariant(CreativeWorkVariant),

    String(String),
}
