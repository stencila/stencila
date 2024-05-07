// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// A description of the provenance of content in terms of human/machine involvement.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ProvenanceCategory {
    /// Content that is human written, edited and verified.
    HwHeHv,

    /// Content that is human written and edited.
    HwHe,

    /// Content that is human written and verified.
    HwHv,

    /// Content that is human written.
    #[default]
    Hw,

    /// Content that is human written, machine verified.
    HwMv,

    /// Content that is machine written, human edited, human verified.
    MwHeHv,

    /// Content that is machine written, human edited.
    MwHe,

    /// Content that is machine written, human edited, machine verified.
    MwHeMv,

    /// Content that is human written, machine edited, human verified.
    HwMeHv,

    /// Content that is human written, machine edited.
    HwMe,

    /// Content that is human written, machine edited, machine verified.
    HwMeMv,

    /// Content that is machine written, human verified.
    MwHv,

    /// Content that is machine written, machine edited, human verified.
    MwMeHv,

    /// Content that is machine written.
    Mw,

    /// Content that is machine written and verified.
    MwMv,

    /// Content that is machine written and edited.
    MwMe,

    /// Content that is machine written, edited and verified.
    MwMeMv,
}
