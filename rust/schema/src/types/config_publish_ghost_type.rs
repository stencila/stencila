// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of Ghost resource
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ConfigPublishGhostType {
    /// A Ghost page
    #[default]
    #[serde(rename = "page")]
    #[serde(alias = "Page")]
    Page,

    /// A Ghost post
    #[serde(rename = "post")]
    #[serde(alias = "Post")]
    Post,
}
