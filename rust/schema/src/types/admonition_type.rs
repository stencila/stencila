// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of an `Admonition`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum AdmonitionType {
    #[default]
    Note,

    Info,

    Tip,

    Important,

    Success,

    Failure,

    Warning,

    Danger,

    Error,
}
