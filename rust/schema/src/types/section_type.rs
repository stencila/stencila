// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a `Section`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, strum::EnumString, ReadNode)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum SectionType {
    Main,

    Header,

    Footer,

    Summary,

    Introduction,

    Methods,

    Results,

    Discussion,

    Conclusion,
}
