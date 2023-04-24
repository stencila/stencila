use crate::prelude::*;

/// The type of a `Claim`.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
#[def = "Statement"]
pub enum ClaimType {
    Statement,
    Theorem,
    Lemma,
    Proof,
    Postulate,
    Hypothesis,
    Proposition,
    Corollary,
}
