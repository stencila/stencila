// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::string::String;

/// [`SoftwareApplication`] or [`SoftwareSourceCode`] or [`String`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
pub enum SoftwareApplicationOrSoftwareSourceCodeOrString {
    #[default]
    SoftwareApplication(SoftwareApplication),

    SoftwareSourceCode(SoftwareSourceCode),

    String(String),
}
