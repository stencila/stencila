// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The kind of evidence supporting a graph edge.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum GraphEvidenceKind {
    /// Evidence from an explicit authored or schema field declaration, such as a link target, citation target, include source, or media URL.
    #[default]
    Declared,

    /// Evidence from deterministic resolution when that resolution is itself the sole evidentiary basis.
    Resolved,

    /// Evidence from direct inspection of existing local state, such as filesystem entries or symlink targets.
    Observed,

    /// Evidence from a deterministic Stencila operation, such as walking document structure, decoding a file, or materializing recorded execution outputs.
    Computed,

    /// Evidence from persisted Stencila metadata already recorded on a node, such as execution dependencies, execution digests, or execution status.
    Recorded,

    /// Evidence inferred by analyzing code or configuration without executing it, rather than copied from an explicit authored relationship field.
    StaticAnalysis,

    /// Evidence from a confirmed operation observed while executing code, such as a successful file access, import, or remote response.
    RuntimeAnalysis,

    /// Evidence imported from an external metadata source, service, graph, or document format.
    Imported,

    /// Evidence explicitly asserted by a user.
    UserAssertion,

    /// Evidence from a signed, cryptographic, or otherwise verifiable attestation such as a content credential.
    Attested,

    /// Evidence from a heuristic or probabilistic inference that has not been directly resolved, observed, or attested.
    Inferred,
}
