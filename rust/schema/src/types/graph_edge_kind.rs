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

    /// The upstream node is called by the downstream node. Approximately related to `prov:used` with inverse direction, or `prov:wasInformedBy` for activity dependencies.
    CalledBy,

    /// The upstream node is derived into the downstream node. Related to `prov:wasDerivedFrom` with inverse direction.
    DerivedInto,

    /// The upstream node is converted into the downstream node, usually changing representation or media format. Related to `prov:wasDerivedFrom` with inverse direction.
    ConvertedInto,

    /// The upstream node parameterizes the downstream node. Approximately related to `prov:used` with inverse direction.
    Parameterizes,

    /// The source node depends on the target node. Used for direct code-unit reactivity dependencies where consumers need the dependent-to-dependency direction.
    DependsOn,

    /// The upstream node is part of the downstream node. Related to `schema:isPartOf` and inverse `schema:hasPart`.
    PartOf,

    /// The upstream node is transcluded by the downstream node. Stencila extension with no exact core PROV-O equivalent; may be derivation-like or inclusion-like depending on context.
    TranscludedBy,

    /// The upstream node is imported by the downstream node. Approximately related to `prov:hadPrimarySource` or `prov:wasDerivedFrom` with inverse direction.
    ImportedBy,

    /// The upstream node is cited by the downstream node. Stencila document relation with no exact core PROV-O equivalent; quotation-specific cases may relate to `prov:wasQuotedFrom` with inverse direction.
    CitedBy,

    /// The upstream node is referenced by the downstream node. Stencila document relation with no exact core PROV-O equivalent.
    ReferencedBy,
}
