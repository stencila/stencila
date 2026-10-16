//! The built-in schema-native policies
//!
//! Alignment decisions and value equality are kept separate:
//!
//! - the alignment policy decides candidate compatibility, feature selection, pair and
//!   gap costs, collection mode, and hard rejection;
//! - the value policy decides projection exclusions, scalar adapters, and scalar
//!   equivalence.
//!
//! Matching normalization and equality normalization are separate concerns: the
//! aligner may normalize text in order to locate a pair, while the value policy still
//! records the original strings as different. Normalization used to find candidates
//! must never suppress a value difference.
//!
//! The first release has one built-in schema-native implementation of each, and no
//! serialized policy language.

use stencila_node_type::NodeType;
use stencila_schema::{PropertyDecl, ValueKind};

use crate::{
    alignment::{AlignmentCost, AlignmentSignal, EvidenceValue, MatchEvidence, PairCost},
    error::CompareResult,
    features::Features,
    person,
    projection::{Item, OccurrenceId, Projection},
    scalar::{CanonicalNumber, ScalarValue},
    text,
};

/// The name of the alignment algorithm
pub const ALGORITHM_NAME: &str = "stencila-schema-native";

/// The version of the alignment algorithm
pub const ALGORITHM_VERSION: &str = "2";

/// The name of the built-in policy
pub const POLICY_NAME: &str = "schema-native";

/// The cost of leaving an occurrence, and everything below it, unmatched
///
/// One unit per structured occurrence in the subtree, so that dropping a large subtree
/// is more costly than dropping a small one, and so that the cost of a gap is
/// independent of anything on the other side. Being a pure function of one projection,
/// it is trivially symmetric under swapping the inputs.
pub fn gap_cost(projection: &Projection, id: OccurrenceId) -> CompareResult<AlignmentCost> {
    Ok(AlignmentCost::ONE.saturating_mul(projection.occurrence(id)?.subtree_size))
}

/// The structured occurrences directly contained by an occurrence, in projection order
pub fn children(projection: &Projection, id: OccurrenceId) -> CompareResult<Vec<OccurrenceId>> {
    Ok(projection
        .occurrence(id)?
        .properties
        .iter()
        .flat_map(|property| property.items.iter())
        .filter_map(|item| match item {
            Item::Structured(id) => Some(*id),
            Item::Scalar(..) => None,
        })
        .collect())
}

/// The weights of the signals that make up the dissimilarity of a candidate pair
///
/// They sum to one, so a dissimilarity is itself between zero and one.
mod weights {
    use crate::alignment::AlignmentCost;

    /// Whether the two occurrences have the same concrete node type
    ///
    /// A quarter, so a cross-type pair whose content is otherwise identical is still
    /// worth making — that is how identical inline content stays visible when a
    /// paragraph becomes a heading — while a cross-type pair whose content also
    /// differs is not.
    pub const NODE_TYPE: i64 = 250;

    /// Similarity of the normalized text of the two subtrees, or of two people's
    /// identifiers and own names
    pub const TEXT: i64 = 650;

    /// Whether the occurrences' own scalar properties agree
    pub const SCALAR_SIGNATURE: i64 = 100;

    /// The scale on which the weights and dissimilarities are expressed
    pub const SCALE: i64 = AlignmentCost::SCALE;
}

/// How much more a pair costs than the dissimilarity it represents
///
/// A gap costs one unit per structured occurrence, so leaving two items unmatched
/// costs the sum of their subtree sizes. Pricing a pair at twice its dissimilarity
/// times that same sum makes the rule exact and easy to state: **two items are paired
/// only when they are more than half similar**. At exactly half similar, the pair ties
/// with the two gaps, and the gaps win, so an implausible candidate is left unmatched
/// as a consequence of the cost model rather than of a cleanup pass.
const PAIR_COST_FACTOR: i64 = 2;

/// Whether two candidate items may be paired at all
///
/// Same-type structured items are compatible by default. Differently typed structured
/// items are compatible only when both are valid for the same schema slot: that is,
/// when the property is declared identically on both sides and its values are a
/// heterogeneous union, so that both types genuinely belong there. Text similarity
/// alone never makes a cross-type pair compatible.
/// A section additionally needs a shared explicit identity to match a non-section;
/// its descendant text is not evidence that the wrapper replaced a content block.
///
/// Structured and scalar items are never compatible with each other.
pub fn compatible(
    left: CandidateKind<'_>,
    right: CandidateKind<'_>,
    left_decl: &PropertyDecl,
    right_decl: &PropertyDecl,
) -> bool {
    match (left, right) {
        (CandidateKind::Structured(left), CandidateKind::Structured(right)) => {
            // Two items of the same type are compatible wherever they are. Two items
            // of different types are compatible only when the property is declared
            // identically on both sides — same cardinality, same union — so that both
            // variants really are valid for the same schema slot.
            left.node_type == right.node_type
                || (left_decl == right_decl
                    && left_decl.kind == ValueKind::Union
                    // A section's text comes from its blocks. Pairing the wrapper
                    // with one of those blocks consumes the block's real counterpart
                    // before cross-parent reconciliation can find it. Only explicit
                    // identity justifies treating a section as a replacement block.
                    && (left.node_type != NodeType::Section && right.node_type != NodeType::Section
                        || left.explicit_id.is_some() && left.explicit_id == right.explicit_id))
        }
        (CandidateKind::Scalar(..), CandidateKind::Scalar(..)) => true,
        _ => false,
    }
}

/// Whether a shared explicit `id` is enough, on its own, to anchor two items
///
/// An ID is normally a deliberate identity. A reference's ID is different: tools
/// commonly number references by position (`b0`, `b1`, …), so after one is dropped the
/// same label names a different work on each side. Two references sharing an ID
/// therefore anchor only when they are also more than half similar, the threshold at
/// which the pair cost would pair them anyway; otherwise they are left to that cost.
pub fn id_corroborated(left: CandidateKind<'_>, right: CandidateKind<'_>) -> bool {
    match (left, right) {
        (CandidateKind::Structured(left), CandidateKind::Structured(right))
            if left.node_type == NodeType::Reference || right.node_type == NodeType::Reference =>
        {
            corroborates(&left.grams, &right.grams)
        }
        _ => true,
    }
}

/// Whether two texts are more than half similar, enough to corroborate a shared label
fn corroborates(left: &text::Grams, right: &text::Grams) -> bool {
    text::similarity(left, right).units() * 2 > weights::SCALE
}

/// Whether two cross-references evidently refer to the same thing
///
/// Two citations that each resolve to a reference are judged by those references'
/// text, whatever their labels, for the reason [`id_corroborated`] gives. Otherwise
/// only an unchanged label is evidence.
fn same_target(left: &Features, right: &Features) -> bool {
    match (&left.cited, &right.cited) {
        (Some(left), Some(right)) => corroborates(left, right),
        (None, None) => {
            left.reference_target.is_some() && left.reference_target == right.reference_target
        }
        _ => false,
    }
}

/// One side of a candidate pair
#[derive(Debug, Clone, Copy)]
pub enum CandidateKind<'features> {
    Structured(&'features Features),
    Scalar(&'features ScalarValue),
}

impl CandidateKind<'_> {
    /// The number of structured occurrences this candidate stands for
    ///
    /// A scalar item is not an occurrence, but it still displaces one position in a
    /// sequence, so it is priced as one.
    pub fn size(&self) -> i64 {
        match self {
            Self::Structured(features) => features.subtree_size,
            Self::Scalar(..) => 1,
        }
    }
}

/// The cost of leaving a candidate item unmatched
pub fn item_gap_cost(candidate: CandidateKind<'_>) -> AlignmentCost {
    AlignmentCost::ONE.saturating_mul(candidate.size())
}

/// The cost of pairing two candidate items, with the evidence that explains it
pub struct Candidate {
    /// The cost, or a refusal to pair
    pub cost: PairCost,

    /// The signals that contributed to the cost
    pub evidence: Vec<MatchEvidence>,
}

/// Score a candidate pair
///
/// Dissimilarity is a weighted sum of alignment-independent signals, so scoring never
/// trial-aligns descendants and never depends on a proposed correspondence.
pub fn pair_cost(
    left: CandidateKind<'_>,
    right: CandidateKind<'_>,
    left_decl: &PropertyDecl,
    right_decl: &PropertyDecl,
) -> Candidate {
    if !compatible(left, right, left_decl, right_decl) {
        return Candidate {
            cost: PairCost::Forbidden,
            evidence: Vec::new(),
        };
    }

    let scale = weights::SCALE;
    let size = left.size().saturating_add(right.size());

    // A dissimilarity is expressed in parts of `scale`, and a cost of one is `scale`
    // units, so the two scales cancel: a dissimilarity of `scale` over a combined size
    // of `size` costs `size` times `PAIR_COST_FACTOR` whole units
    let units =
        |parts: i64| AlignmentCost::from_units(parts.saturating_mul(size * PAIR_COST_FACTOR));

    let mut evidence = Vec::new();
    let mut dissimilarity = 0i64;

    match (left, right) {
        (CandidateKind::Structured(left), CandidateKind::Structured(right)) => {
            let same_type = left.node_type == right.node_type;
            let node_type = if same_type { 0 } else { scale };
            evidence.push(MatchEvidence {
                signal: AlignmentSignal::NodeType,
                value: EvidenceValue::Boolean { value: same_type },
                contribution: units(node_type * weights::NODE_TYPE / scale),
            });

            let subtree = text::similarity(&left.grams, &right.grams);
            // A person and a role are left to subtree text: pairing them by the person
            // would leave the person within the role unmatched
            let (signal, similarity) = match (&left.person, &right.person) {
                // The same target keeps a citation paired when its rendering
                // appears or changes, by closing half of the remaining text
                // dissimilarity. It is a bonus rather than a replacement for text, so
                // that among several references to the same target the one whose
                // text is closest still wins. A different target contributes no
                // penalty: target equality is decided through the completed alignment.
                _ if same_type && same_target(left, right) => (
                    AlignmentSignal::ReferenceTarget,
                    AlignmentCost::from_units(
                        subtree.units() + (AlignmentCost::ONE.units() - subtree.units()) / 2,
                    ),
                ),
                (Some(left), Some(right)) if same_type => (
                    AlignmentSignal::PersonIdentity,
                    person::similarity(left, right, subtree),
                ),
                _ => (AlignmentSignal::TextSimilarity, subtree),
            };
            let text = scale - similarity.units();
            evidence.push(MatchEvidence {
                signal,
                value: EvidenceValue::Number {
                    value: CanonicalNumber::new(similarity.units() as f64 / scale as f64),
                },
                contribution: units(text * weights::TEXT / scale),
            });

            let same_scalars = left.scalar_signature == right.scalar_signature;
            let scalars = if same_scalars { 0 } else { scale };
            evidence.push(MatchEvidence {
                signal: AlignmentSignal::ScalarSignature,
                value: EvidenceValue::Boolean {
                    value: same_scalars,
                },
                contribution: units(scalars * weights::SCALAR_SIGNATURE / scale),
            });

            dissimilarity = (node_type * weights::NODE_TYPE
                + text * weights::TEXT
                + scalars * weights::SCALAR_SIGNATURE)
                / scale;
        }
        (CandidateKind::Scalar(left), CandidateKind::Scalar(right)) => {
            // Two scalar items are the same value, or they are compared as text so
            // that a small edit still anchors the structure around it
            let similarity = scalar_similarity(left, right);
            dissimilarity = scale - similarity.units();
            evidence.push(MatchEvidence {
                signal: AlignmentSignal::ScalarSignature,
                value: EvidenceValue::Number {
                    value: CanonicalNumber::new(similarity.units() as f64 / scale as f64),
                },
                contribution: units(dissimilarity),
            });
        }
        _ => {}
    }

    Candidate {
        cost: PairCost::Cost(units(dissimilarity)),
        evidence,
    }
}

/// The similarity of two scalar items of a mixed collection
fn scalar_similarity(left: &ScalarValue, right: &ScalarValue) -> AlignmentCost {
    if left == right {
        return AlignmentCost::ONE;
    }

    match (left, right) {
        (ScalarValue::String { value: left }, ScalarValue::String { value: right }) => {
            let left = text::Grams::new(&text::normalize(left));
            let right = text::Grams::new(&text::normalize(right));
            text::similarity(&left, &right)
        }
        _ => AlignmentCost::ZERO,
    }
}
