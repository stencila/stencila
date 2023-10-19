// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a `Claim`.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(crate = "common::serde")]
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
