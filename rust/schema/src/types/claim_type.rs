//! Generated file, do not edit

use crate::prelude::*;

/// The type of a `Claim`.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
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
