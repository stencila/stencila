// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// A description of the provenance of content in terms of human/machine involvement.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ProvenanceCategory {
    /// Human written, edited and verified.
    HwHeHv,

    /// Human written and edited.
    HwHe,

    /// Human written and verified.
    HwHv,

    /// Human written.
    #[default]
    Hw,

    /// Human written, machine verified.
    HwMv,

    /// Machine written, human edited, human verified.
    MwHeHv,

    /// Machine written, human edited.
    MwHe,

    /// Machine written, human edited, machine verified.
    MwHeMv,

    /// Human written, machine edited, human verified.
    HwMeHv,

    /// Human written, machine edited.
    HwMe,

    /// Human written, machine edited, machine verified.
    HwMeMv,

    /// Machine written, human verified.
    MwHv,

    /// Machine written, machine edited, human verified.
    MwMeHv,

    /// Machine written.
    Mw,

    /// Machine written and verified.
    MwMv,

    /// Machine written and edited.
    MwMe,

    /// Machine written, edited and verified.
    MwMeMv,
}
