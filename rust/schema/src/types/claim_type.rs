// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a `Claim`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, MergeNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
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
