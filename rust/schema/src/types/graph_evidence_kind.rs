// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The kind of evidence supporting a graph edge.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum GraphEvidenceKind {
    /// Evidence from static analysis.
    #[default]
    StaticAnalysis,

    /// Evidence from observing runtime behavior.
    RuntimeObservation,

    /// Evidence asserted by a user.
    UserAssertion,

    /// Evidence imported from another source.
    Imported,

    /// Evidence from a content credential.
    ContentCredential,

    /// Evidence from execution metadata.
    ExecutionMetadata,
}
