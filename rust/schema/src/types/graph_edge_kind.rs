// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The kind of directed relationship represented by a graph edge.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum GraphEdgeKind {
    /// The upstream node is used by the downstream node. Related to `prov:used` with inverse direction.
    #[default]
    UsedBy,

    /// The upstream node is read by the downstream node. Related to `prov:used` with inverse direction.
    ReadBy,

    /// The upstream node generated the downstream node. Related to `prov:generated` and to `prov:wasGeneratedBy` with inverse direction.
    Generated,

    /// The upstream data or value is written to the downstream resource. Related to `prov:generated` for the downstream resource and to `prov:used` for the upstream value.
    WrittenTo,

    /// The upstream node is derived into the downstream node. Related to `prov:wasDerivedFrom` with inverse direction.
    DerivedInto,

    /// The upstream node is converted into the downstream node, usually changing representation or media format. Related to `prov:wasDerivedFrom` with inverse direction.
    ConvertedInto,

    /// The upstream node is called by the downstream node. Approximately related to `prov:used` with inverse direction, or `prov:wasInformedBy` for activity dependencies.
    CalledBy,

    /// The upstream node is imported by the downstream node. Approximately related to `prov:hadPrimarySource` or `prov:wasDerivedFrom` with inverse direction.
    ImportedBy,

    /// The upstream node is part of the downstream node. Related to `schema:isPartOf` and inverse `schema:hasPart`.
    PartOf,

    /// The upstream source is included by the downstream document node or document region.
    IncludedBy,

    /// The upstream resource is linked to by the downstream link, media object, document, or document region.
    LinkedBy,

    /// The upstream node is cited by the downstream node. Stencila document relation with no exact core PROV-O equivalent; quotation-specific cases may relate to `prov:wasQuotedFrom` with inverse direction.
    CitedBy,

    /// The upstream manifest, configuration, or source file declares the downstream environment, dependency, workflow, or other computational resource.
    Declares,

    /// The upstream configuration file or setting configures the downstream workflow, environment, code unit, or tool.
    Configures,

    /// The upstream package or software component is required by the downstream environment or source code.
    RequiredBy,

    /// The upstream lockfile, digest, or exact version pin constrains the downstream environment or dependency.
    Pins,

    /// The source research object, typically evidence or a claim, supports the target claim.
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
