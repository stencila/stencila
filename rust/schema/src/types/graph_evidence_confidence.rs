// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The confidence level for graph evidence.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum GraphEvidenceConfidence {
    /// Plausible but weakly supported or not independently verified.
    Low,

    /// Supported by one reliable signal, but not enough to treat as definitive.
    Medium,

    /// Supported by multiple corroborating signals or a deterministic local check.
    High,

    /// Directly observed, deterministically proven, or cryptographically attested with no known contrary evidence.
    #[default]
    Certain,
}
