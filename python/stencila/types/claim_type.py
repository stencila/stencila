# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ClaimType(StrEnum):
    """
    The type of a `Claim`.
    """

    Statement = "Statement"
    Theorem = "Theorem"
    Lemma = "Lemma"
    Proof = "Proof"
    Postulate = "Postulate"
    Hypothesis = "Hypothesis"
    Proposition = "Proposition"
    Corollary = "Corollary"
