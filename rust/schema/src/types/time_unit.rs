// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// A unit in which time can be measured.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum TimeUnit {
    Year,

    Month,

    Week,

    Day,

    Hour,

    Minute,

    Second,

    #[default]
    Millisecond,

    Microsecond,

    Nanosecond,

    Picosecond,

    Femtosecond,

    Attosecond,
}
