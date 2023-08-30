// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::creative_work_type::CreativeWorkType;
use super::string::String;

/// [`CreativeWorkType`] or [`String`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, Read, Write)]
#[serde(untagged, crate = "common::serde")]
pub enum CreativeWorkTypeOrString {
    CreativeWorkType(CreativeWorkType),
    String(String),
}
