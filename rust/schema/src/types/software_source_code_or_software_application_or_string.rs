// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::string::String;

/// [`SoftwareSourceCode`] or [`SoftwareApplication`] or [`String`]
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, Read, Write)]
#[serde(untagged, crate = "common::serde")]
pub enum SoftwareSourceCodeOrSoftwareApplicationOrString {
    SoftwareSourceCode(SoftwareSourceCode),
    SoftwareApplication(SoftwareApplication),
    String(String),
}
