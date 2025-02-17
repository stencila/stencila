// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The state of Ghost resource
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ConfigPublishGhostState {
    /// a draft page or post.
    #[default]
    #[serde(rename = "draft")]
    #[serde(alias = "Draft")]
    Draft,

    /// Publish pagor or post.
    #[serde(rename = "publish")]
    #[serde(alias = "Publish")]
    Publish,
}
