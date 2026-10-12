// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Distinct evidence availability and coverage limitations.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum GraphGapState {
    /// The evidence or property is unavailable or was never recorded.
    #[default]
    Missing,

    /// The material was removed by ordinary retention.
    Expired,

    /// Authority or disclosure policy prevents returning the material.
    Withheld,

    /// This source did not observe the operation or relationship.
    Unobserved,

    /// The owner explicitly removed the retained material.
    Purged,

    /// Inspection or projection stopped at a budget, without claiming availability.
    Omitted,
}
