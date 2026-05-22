// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::container_image::ContainerImage;
use super::string::String;

/// [`ContainerImage`] or [`String`]
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
pub enum ContainerImageOrString {
    #[default]
    ContainerImage(ContainerImage),

    String(String),
}
