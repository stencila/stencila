// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a `Claim`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ClaimType {
    #[default]
    Statement,

    Theorem,

    Lemma,

    Proof,

    Postulate,

    Hypothesis,

    Proposition,

    Corollary,
}
