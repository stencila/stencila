//! The alignment artifact
//!
//! An alignment answers which occurrences correspond. It does not say whether their
//! types, positions, or properties are equal — that is what a comparison adds.
//!
//! Alignment is a flat relation over occurrences, not a merged tree. The two inputs
//! retain their separate hierarchies, so a consumer may derive a left-oriented,
//! right-oriented or combined view without either hierarchy being baked into the
//! foundational model.

use std::{cmp::Ordering, collections::HashSet};

use serde::{Deserialize, Serialize};

use stencila_node_path::NodePath;
use stencila_node_type::NodeType;

use crate::{
    error::{CompareError, CompareResult, Side},
    projection::Projection,
    scalar::CanonicalNumber,
};

/// The version of the alignment format
///
/// Versioned independently of the comparison format, because an alignment is a
/// deliverable in its own right.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlignmentFormatVersion {
    #[default]
    #[serde(rename = "1")]
    V1,
}

/// Which algorithm, projection and policy produced an artifact
///
/// Recorded so that two artifacts can be told apart when any of them changes, and so
/// that a future approximate algorithm is identifiable rather than silently
/// substituted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlgorithmInfo {
    /// The name of the alignment algorithm
    pub name: String,

    /// The version of the alignment algorithm
    pub version: String,

    /// The version of the canonical projection
    pub projection_version: String,

    /// The name of the built-in policy
    pub policy: String,
}

/// A reference to a structured occurrence in one of the inputs
///
/// A [`NodePath`] plus a [`NodeType`] is the canonical occurrence reference. Internal
/// node UIDs are unsuitable, because independently decoded trees receive unrelated
/// UIDs and Stencila deliberately excludes them from node equality. An explicit schema
/// `id` property remains an ordinary compared value, and a possible matching signal.
///
/// Paths refer to the immutable input snapshots. They are not promised to survive
/// later mutation of either input.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeRef {
    /// The path to the occurrence from the root of its input
    pub path: NodePath,

    /// The concrete node type of the occurrence
    pub node_type: NodeType,
}

impl NodeRef {
    /// Create a node reference
    pub fn new(path: NodePath, node_type: NodeType) -> Self {
        Self { path, node_type }
    }
}

/// A cost used when selecting an alignment
///
/// Costs are fixed-point integers rather than accumulated floating point values, so
/// that the same inputs, options and algorithm version produce byte-for-byte
/// equivalent artifacts. A cost is a policy-specific quantity, not a probability or a
/// calibrated confidence.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct AlignmentCost(i64);

impl AlignmentCost {
    /// The number of units in a cost of one
    pub const SCALE: i64 = 1_000;

    /// A cost of zero
    pub const ZERO: Self = Self(0);

    /// A cost of one
    pub const ONE: Self = Self(Self::SCALE);

    /// Create a cost from its fixed-point units
    pub const fn from_units(units: i64) -> Self {
        Self(units)
    }

    /// Create a cost from a ratio, rounding towards zero
    ///
    /// Used instead of floating point arithmetic so that costs derived from
    /// similarity measures remain exactly reproducible.
    pub const fn from_ratio(numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            return Self::ZERO;
        }
        Self(numerator.saturating_mul(Self::SCALE) / denominator)
    }

    /// The fixed-point units of this cost
    pub const fn units(self) -> i64 {
        self.0
    }

    /// Add another cost, saturating rather than overflowing
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Multiply by an integer, saturating rather than overflowing
    pub const fn saturating_mul(self, factor: i64) -> Self {
        Self(self.0.saturating_mul(factor))
    }
}

/// The cost of pairing two candidates, or a refusal to pair them at all
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PairCost {
    /// The candidates may be paired, at this cost
    Cost(AlignmentCost),

    /// The candidates are incompatible and must not be paired
    Forbidden,
}

/// The rule that selected a pair
///
/// These are the reasons a pair exists, not a description of how the two occurrences
/// differ: a pair may be selected by any of these rules and still be cross-type,
/// reordered, moved and value-modified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MatchRule {
    /// The two caller-selected roots
    Root,

    /// Both occurrences are the value of the same singular property of a paired parent
    SingularProperty,

    /// Both occurrences carry the same explicit schema `id`, uniquely within the scope
    UniqueId,

    /// Both occurrences have the same fingerprint, verified against their projected
    /// values, uniquely within the scope
    VerifiedExactFingerprint,

    /// The pair was selected by order-preserving alignment of a repeated property
    SequenceAlignment,

    /// The pair was selected by reconciliation across different parents
    CrossParentReconciliation,
}

/// Why an occurrence has no counterpart
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnmatchedReason {
    /// Nothing on the other side was a compatible candidate
    NoCompatibleCandidate,

    /// There were compatible candidates, but leaving both items unmatched cost less
    /// than every one of them
    GapCheaperThanPair,
}

/// A signal that contributed to selecting a pair
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AlignmentSignal {
    /// Whether the two occurrences have the same concrete node type
    NodeType,

    /// A non-empty explicit schema `id`
    ExplicitId,

    /// The canonical structural fingerprint of the subtree
    CanonicalFingerprint,

    /// The structural fingerprint of the subtree, excluding explicit `id`
    IdentityNeutralFingerprint,

    /// The shallow signature of an occurrence's scalar properties
    ScalarSignature,

    /// Similarity of normalized descendant text
    TextSimilarity,

    /// The original position of an occurrence within its sibling scope
    Position,
}

/// The value of a signal
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum EvidenceValue {
    /// The signal was not available for this pair
    Absent,

    Boolean {
        value: bool,
    },

    Integer {
        value: i64,
    },

    Number {
        value: CanonicalNumber,
    },

    String {
        value: String,
    },
}

/// One piece of evidence for a selected pair
///
/// Evidence explains a selected outcome. The matrix of *rejected* candidates is
/// deliberately not retained, because it can be quadratic; a later opt-in trace
/// facility may capture it for debugging.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchEvidence {
    /// The signal
    pub signal: AlignmentSignal,

    /// The value of the signal for this pair
    pub value: EvidenceValue,

    /// What the signal contributed to the pair cost
    pub contribution: AlignmentCost,
}

impl MatchEvidence {
    /// Invert this evidence, for when the two inputs are swapped
    ///
    /// Every built-in signal is symmetric in the two sides, so evidence inverts to
    /// itself; the method exists so that an asymmetric signal cannot be added without
    /// the inversion being considered.
    pub fn invert(self) -> Self {
        self
    }
}

/// Why a pair was selected, and at what cost
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchInfo {
    /// The rule that selected the pair
    pub rule: MatchRule,

    /// The cost of pairing the two occurrences
    pub pair_cost: AlignmentCost,

    /// The cost that would have been paid to leave the left occurrence unmatched
    pub left_gap_cost: AlignmentCost,

    /// The cost that would have been paid to leave the right occurrence unmatched
    pub right_gap_cost: AlignmentCost,

    /// The signals that contributed to the pair cost
    pub evidence: Vec<MatchEvidence>,
}

impl MatchInfo {
    /// Invert this information, for when the two inputs are swapped
    pub fn invert(self) -> Self {
        Self {
            rule: self.rule,
            pair_cost: self.pair_cost,
            left_gap_cost: self.right_gap_cost,
            right_gap_cost: self.left_gap_cost,
            evidence: self
                .evidence
                .into_iter()
                .map(MatchEvidence::invert)
                .collect(),
        }
    }
}

/// One correspondence between the two inputs
///
/// Deliberately not a single status enum containing values such as matched, moved,
/// type-mismatch or modified: those facts are orthogonal, and one pair may be
/// cross-type, reordered, moved between parents and value-modified all at once. They
/// are reported as separate differences in a comparison.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Correspondence {
    /// Two occurrences correspond
    Paired {
        left: NodeRef,
        right: NodeRef,
        match_info: MatchInfo,
    },

    /// An occurrence of the left input has no counterpart
    LeftOnly {
        left: NodeRef,
        reason: UnmatchedReason,

        /// The nearest ancestor of this occurrence that is also one-sided
        ///
        /// `None` when this occurrence is itself the root of a one-sided subtree.
        /// Lets a presentation layer collapse a one-sided subtree, even though every
        /// structured descendant has its own record.
        nearest_one_sided_ancestor: Option<NodeRef>,
    },

    /// An occurrence of the right input has no counterpart
    RightOnly {
        right: NodeRef,
        reason: UnmatchedReason,

        /// The nearest ancestor of this occurrence that is also one-sided
        nearest_one_sided_ancestor: Option<NodeRef>,
    },
}

impl Correspondence {
    /// The left occurrence, if any
    pub fn left(&self) -> Option<&NodeRef> {
        match self {
            Self::Paired { left, .. } | Self::LeftOnly { left, .. } => Some(left),
            Self::RightOnly { .. } => None,
        }
    }

    /// The right occurrence, if any
    pub fn right(&self) -> Option<&NodeRef> {
        match self {
            Self::Paired { right, .. } | Self::RightOnly { right, .. } => Some(right),
            Self::LeftOnly { .. } => None,
        }
    }

    /// The rank of this kind of correspondence, used as the final canonical sort key
    fn kind_rank(&self) -> u8 {
        match self {
            Self::Paired { .. } => 0,
            Self::LeftOnly { .. } => 1,
            Self::RightOnly { .. } => 2,
        }
    }

    /// The key used to canonically order correspondences
    ///
    /// Correspondences sort by left path when present, then right path, then relation
    /// kind, using typed path slots rather than rendered path strings. A `None` path
    /// sorts before any `Some` path, so right-only records sort before the paired and
    /// left-only records that share their right path.
    fn sort_key(&self) -> (Option<&NodePath>, Option<&NodePath>, u8) {
        (
            self.left().map(|node| &node.path),
            self.right().map(|node| &node.path),
            self.kind_rank(),
        )
    }

    /// The key used to break ties in canonical order
    ///
    /// Two correspondences cannot share both a left and a right path, because every
    /// occurrence is covered exactly once, so this only ever settles comparisons that
    /// the sort key already decided. It exists so that `Ord` agrees with `Eq`: without
    /// it, two unequal records could compare equal, and canonical order would silently
    /// depend on the stability of the sort.
    fn tie_key(
        &self,
    ) -> (
        Option<&MatchInfo>,
        Option<UnmatchedReason>,
        Option<&NodeRef>,
    ) {
        match self {
            Self::Paired { match_info, .. } => (Some(match_info), None, None),
            Self::LeftOnly {
                reason,
                nearest_one_sided_ancestor,
                ..
            }
            | Self::RightOnly {
                reason,
                nearest_one_sided_ancestor,
                ..
            } => (None, Some(*reason), nearest_one_sided_ancestor.as_ref()),
        }
    }

    /// Invert this correspondence, for when the two inputs are swapped
    pub fn invert(self) -> Self {
        match self {
            Self::Paired {
                left,
                right,
                match_info,
            } => Self::Paired {
                left: right,
                right: left,
                match_info: match_info.invert(),
            },
            Self::LeftOnly {
                left,
                reason,
                nearest_one_sided_ancestor,
            } => Self::RightOnly {
                right: left,
                reason,
                nearest_one_sided_ancestor,
            },
            Self::RightOnly {
                right,
                reason,
                nearest_one_sided_ancestor,
            } => Self::LeftOnly {
                left: right,
                reason,
                nearest_one_sided_ancestor,
            },
        }
    }
}

impl PartialOrd for Correspondence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Correspondence {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key()
            .cmp(&other.sort_key())
            .then_with(|| self.tie_key().cmp(&other.tie_key()))
            .then_with(|| match (self, other) {
                (
                    Self::Paired {
                        left,
                        right,
                        match_info,
                    },
                    Self::Paired {
                        left: other_left,
                        right: other_right,
                        match_info: other_match_info,
                    },
                ) => (left, right, match_info).cmp(&(other_left, other_right, other_match_info)),
                (
                    Self::LeftOnly {
                        left,
                        reason,
                        nearest_one_sided_ancestor,
                    },
                    Self::LeftOnly {
                        left: other_left,
                        reason: other_reason,
                        nearest_one_sided_ancestor: other_ancestor,
                    },
                ) => (left, reason, nearest_one_sided_ancestor).cmp(&(
                    other_left,
                    other_reason,
                    other_ancestor,
                )),
                (
                    Self::RightOnly {
                        right,
                        reason,
                        nearest_one_sided_ancestor,
                    },
                    Self::RightOnly {
                        right: other_right,
                        reason: other_reason,
                        nearest_one_sided_ancestor: other_ancestor,
                    },
                ) => (right, reason, nearest_one_sided_ancestor).cmp(&(
                    other_right,
                    other_reason,
                    other_ancestor,
                )),
                _ => Ordering::Equal,
            })
    }
}

/// A complete, symmetric correspondence between the structured occurrences of two
/// nodes
///
/// Every structured occurrence on both sides appears exactly once: either in one
/// paired correspondence, or in one one-sided correspondence. No occurrence is paired
/// more than once.
///
/// Deserialization restores canonical order and rejects duplicate paths. Call
/// [`Alignment::validate`] with the original snapshots before trusting a deserialized
/// artifact's path resolution and complete coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "AlignmentData")]
pub struct Alignment {
    /// The version of the alignment format
    format_version: AlignmentFormatVersion,

    /// The algorithm, projection and policy that produced this alignment
    algorithm: AlgorithmInfo,

    /// The correspondences, in canonical order
    correspondences: Vec<Correspondence>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlignmentData {
    format_version: String,
    algorithm: AlgorithmInfo,
    correspondences: Vec<Correspondence>,
}

impl TryFrom<AlignmentData> for Alignment {
    type Error = CompareError;

    fn try_from(data: AlignmentData) -> Result<Self, Self::Error> {
        let format_version = match data.format_version.as_str() {
            "1" => AlignmentFormatVersion::V1,
            version => {
                return Err(CompareError::UnsupportedVersion {
                    artifact: "alignment",
                    version: version.to_string(),
                });
            }
        };
        let mut alignment = Self {
            format_version,
            algorithm: data.algorithm,
            correspondences: data.correspondences,
        };
        alignment.canonicalize();
        alignment.validate_local()?;
        Ok(alignment)
    }
}

impl Alignment {
    /// Create an alignment, putting its correspondences into canonical order
    pub(crate) fn new(
        algorithm: AlgorithmInfo,
        correspondences: Vec<Correspondence>,
    ) -> CompareResult<Self> {
        let mut alignment = Self {
            format_version: AlignmentFormatVersion::V1,
            algorithm,
            correspondences,
        };
        alignment.canonicalize();
        alignment.validate_local()?;
        Ok(alignment)
    }

    /// The version of the alignment format
    pub fn format_version(&self) -> AlignmentFormatVersion {
        self.format_version
    }

    /// The algorithm, projection and policy that produced this alignment
    pub fn algorithm(&self) -> &AlgorithmInfo {
        &self.algorithm
    }

    /// The correspondences, in canonical order
    pub fn correspondences(&self) -> &[Correspondence] {
        &self.correspondences
    }

    /// Put the correspondences into canonical order
    ///
    /// Canonical ordering is part of the format contract: it holds in memory as well
    /// as after deserialization, and inversion re-canonicalizes.
    fn canonicalize(&mut self) {
        self.correspondences.sort();
    }

    /// Validate the references and complete, single coverage against two snapshots
    pub fn validate(
        &self,
        left: &stencila_schema::Node,
        right: &stencila_schema::Node,
    ) -> CompareResult<()> {
        self.validate_local()?;
        let left = Projection::new(left, Side::Left)?;
        let right = Projection::new(right, Side::Right)?;
        self.validate_projection(&left)?;
        self.validate_projection(&right)
    }

    /// Validate invariants that do not require the original snapshots
    fn validate_local(&self) -> CompareResult<()> {
        let mut left_paths = HashSet::new();
        let mut right_paths = HashSet::new();
        for correspondence in &self.correspondences {
            if let Some(left) = correspondence.left()
                && !left_paths.insert(&left.path)
            {
                return Err(CompareError::Uniqueness {
                    side: Side::Left,
                    path: left.path.clone(),
                });
            }
            if let Some(right) = correspondence.right()
                && !right_paths.insert(&right.path)
            {
                return Err(CompareError::Uniqueness {
                    side: Side::Right,
                    path: right.path.clone(),
                });
            }
        }
        Ok(())
    }

    fn validate_projection(&self, projection: &Projection) -> CompareResult<()> {
        let side = projection.side();
        let references: Vec<_> = self
            .correspondences
            .iter()
            .filter_map(|correspondence| match side {
                Side::Left => correspondence.left(),
                Side::Right => correspondence.right(),
            })
            .collect();

        // A scalar root has no projected occurrence, but still has exactly one root
        // reference in an alignment.
        if projection.occurrences().is_empty() {
            let valid = references.len() == 1
                && references[0].path == NodePath::new()
                && references[0].node_type == projection.root_node_type()?;
            if valid {
                return Ok(());
            }
            return Err(CompareError::PathResolution {
                side,
                path: NodePath::new(),
                expected: projection.root_node_type()?,
            });
        }

        if references.len() != projection.occurrences().len() {
            return Err(CompareError::Completeness {
                side,
                covered: references.len(),
                projected: projection.occurrences().len(),
            });
        }

        let resolvable: HashSet<_> = references
            .iter()
            .map(|reference| (&reference.path, reference.node_type))
            .collect();
        for occurrence in projection.occurrences() {
            if !resolvable.contains(&(&occurrence.path, occurrence.node_type)) {
                return Err(CompareError::PathResolution {
                    side,
                    path: occurrence.path.clone(),
                    expected: occurrence.node_type,
                });
            }
        }

        Ok(())
    }

    /// Invert this alignment, as though the two inputs had been swapped
    ///
    /// Swapping the inputs and inverting the result yields the same canonical
    /// artifact.
    pub fn invert(self) -> Self {
        let mut inverted = Self {
            format_version: self.format_version,
            algorithm: self.algorithm,
            correspondences: self
                .correspondences
                .into_iter()
                .map(Correspondence::invert)
                .collect(),
        };
        inverted.canonicalize();
        inverted
    }

    /// The paired correspondences
    pub fn pairs(&self) -> impl Iterator<Item = (&NodeRef, &NodeRef, &MatchInfo)> {
        self.correspondences
            .iter()
            .filter_map(|correspondence| match correspondence {
                Correspondence::Paired {
                    left,
                    right,
                    match_info,
                } => Some((left, right, match_info)),
                _ => None,
            })
    }

    /// Whether any occurrence is one-sided
    pub fn has_one_sided(&self) -> bool {
        self.correspondences.iter().any(|correspondence| {
            matches!(
                correspondence,
                Correspondence::LeftOnly { .. } | Correspondence::RightOnly { .. }
            )
        })
    }
}
