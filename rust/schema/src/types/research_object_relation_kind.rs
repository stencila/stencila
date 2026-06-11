// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The kind of relation from one research object to another.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum ResearchObjectRelationKind {
    /// The source research object, typically evidence or a claim, supports the target claim.
    #[default]
    Supports,

    /// The source claim is supported by the target research object, typically evidence or another claim. Inverse of `Supports`.
    SupportedBy,

    /// The source research object, typically evidence or a claim, opposes the target claim.
    Opposes,

    /// The source claim is opposed by the target research object, typically evidence or another claim. Inverse of `Opposes`.
    OpposedBy,

    /// The source claim addresses the target research question.
    Addresses,

    /// The source research question is addressed by the target claim. Inverse of `Addresses`.
    AddressedBy,

    /// The source research object, typically a study or other research activity, was conducted following the target protocol.
    Follows,

    /// The source research object, typically a study or other research activity, produced and grounds the target evidence.
    Grounds,

    /// The source evidence is grounded in the target study or other research activity that produced it. Inverse of `Grounds`.
    IsGroundedIn,

    /// The source request asks for the target work, such as a study or protocol execution, to be carried out. Not the inverse of `RequestTarget`.
    RequestFor,

    /// The source request concerns the target claim that the requested work may elucidate. Not the inverse of `RequestFor`.
    RequestTarget,
}
