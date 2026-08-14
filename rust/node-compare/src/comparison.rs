//! The comparison artifact
//!
//! A comparison is an alignment plus sparse, atomic observations about how paired
//! occurrences differ.
//!
//! The difference kinds are orthogonal rather than exclusive: one pair may be
//! cross-type, reordered, moved between parents and value-modified all at once, and
//! each of those facts is recorded separately. Locations are typed occurrence
//! references, paths and properties only — no free-form descriptions, semantic paths,
//! or presentation paths.
//!
//! Differences are not metrics. Averaging or aggregating them is a reduction over this
//! factual artifact, and which reduction is wanted varies by use case, so no reduction
//! belongs here.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use stencila_node_path::NodePath;
use stencila_node_type::NodeProperty;

use crate::{
    alignment::{AlgorithmInfo, Alignment, NodeRef},
    scalar::ScalarValue,
};

/// The version of the comparison format
///
/// Versioned independently of the alignment format, because an alignment is also a
/// deliverable in its own right.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComparisonFormatVersion {
    #[default]
    #[serde(rename = "1")]
    V1,
}

/// The state of a property on one side of a pair
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropertyPresence {
    /// The schema does not declare this property for this node type
    ///
    /// Only possible for a cross-type pair, where the two types have a property union
    /// rather than a single property list.
    Undeclared,

    /// The schema declares the property, and it is not present
    ///
    /// Distinct from a present property holding an empty sequence: `None` is absent,
    /// `Some(empty)` is present.
    Absent,

    /// The property is present
    Present,
}

/// Where a value difference is
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueLocation {
    /// The left occurrence whose value changed
    pub left: NodeRef,

    /// The right occurrence whose value changed
    pub right: NodeRef,

    /// The property that holds the value
    ///
    /// `None` for the root value change of two same-type primitive roots, which have
    /// no containing property.
    pub property: Option<NodeProperty>,

    /// The position of the value within a repeated property on the left
    ///
    /// `None` for a singular property, for a homogeneous repeated scalar property
    /// whose whole sequence is one atomic value, and when the value is absent on the
    /// left.
    pub left_index: Option<usize>,

    /// The position of the value within a repeated property on the right
    pub right_index: Option<usize>,
}

/// The value on one side of a value difference
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ValueState {
    /// The value is not present
    Absent,

    /// A single value
    One { value: ScalarValue },

    /// A complete sequence of values
    ///
    /// A homogeneous repeated scalar property is one atomic difference carrying both
    /// complete typed sequences, rather than one difference per item.
    Many { values: Vec<ScalarValue> },
}

/// One atomic observation about how two paired occurrences differ
///
/// Deliberately not a single status enum: these facts are orthogonal, and a pair may
/// exhibit several at once.
///
/// One-sided correspondence is never duplicated as a difference. Property differences
/// are emitted only for paired occurrences, because otherwise every missing subtree
/// would be represented repeatedly at its root, at its descendants, and at its leaves,
/// when the exhaustive one-sided records in the alignment already capture it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Difference {
    /// A constrained cross-type pair
    ///
    /// The shared properties whose projected shapes are compatible are still compared
    /// recursively, so that identical content stays visible when, say, a paragraph
    /// becomes a heading.
    NodeTypeChanged { left: NodeRef, right: NodeRef },

    /// A property that is declared, absent or present differently on the two sides
    PropertyPresenceChanged {
        left: NodeRef,
        right: NodeRef,
        property: NodeProperty,
        left_presence: PropertyPresence,
        right_presence: PropertyPresence,
    },

    /// A changed scalar value
    ///
    /// Atomic: the difference stops at the smallest declared non-structural schema
    /// property. Character, token and text edit scripts are derived presentation data
    /// and do not belong here.
    ValueChanged {
        location: ValueLocation,
        left: ValueState,
        right: ValueState,
    },

    /// A pair whose aligned parents, or containing properties, differ
    ParentChanged {
        left: NodeRef,
        right: NodeRef,
        left_parent: Option<NodeRef>,
        right_parent: Option<NodeRef>,
        left_property: Option<NodeProperty>,
        right_property: Option<NodeProperty>,
    },

    /// A pair whose relative position within its aligned sibling scope changed
    ///
    /// An observation about two snapshots, not a claim about historical editing: it
    /// means the pair lies outside the canonical preserved-order subset of its scope.
    Reordered {
        left: NodeRef,
        right: NodeRef,
        left_scope: Option<NodeRef>,
        right_scope: Option<NodeRef>,
        property: NodeProperty,
    },
}

impl Difference {
    /// The left occurrence this difference is about
    pub fn left(&self) -> &NodeRef {
        match self {
            Self::NodeTypeChanged { left, .. }
            | Self::PropertyPresenceChanged { left, .. }
            | Self::ParentChanged { left, .. }
            | Self::Reordered { left, .. } => left,
            Self::ValueChanged { location, .. } => &location.left,
        }
    }

    /// The right occurrence this difference is about
    pub fn right(&self) -> &NodeRef {
        match self {
            Self::NodeTypeChanged { right, .. }
            | Self::PropertyPresenceChanged { right, .. }
            | Self::ParentChanged { right, .. }
            | Self::Reordered { right, .. } => right,
            Self::ValueChanged { location, .. } => &location.right,
        }
    }

    /// The property this difference is about, if any
    pub fn property(&self) -> Option<NodeProperty> {
        match self {
            Self::NodeTypeChanged { .. } | Self::ParentChanged { .. } => None,
            Self::PropertyPresenceChanged { property, .. } | Self::Reordered { property, .. } => {
                Some(*property)
            }
            Self::ValueChanged { location, .. } => location.property,
        }
    }

    /// The rank of this kind of difference, used in canonical order
    fn kind_rank(&self) -> u8 {
        match self {
            Self::NodeTypeChanged { .. } => 0,
            Self::PropertyPresenceChanged { .. } => 1,
            Self::ValueChanged { .. } => 2,
            Self::ParentChanged { .. } => 3,
            Self::Reordered { .. } => 4,
        }
    }

    /// The key used to canonically order differences
    ///
    /// By left subject and location, then right subject and location, then difference
    /// kind and property, using typed path slots rather than rendered path strings.
    fn sort_key(
        &self,
    ) -> (
        &NodePath,
        &NodePath,
        u8,
        Option<NodeProperty>,
        Option<usize>,
    ) {
        (
            &self.left().path,
            &self.right().path,
            self.kind_rank(),
            self.property(),
            match self {
                Self::ValueChanged { location, .. } => location.left_index,
                _ => None,
            },
        )
    }

    /// Invert this difference, for when the two inputs are swapped
    pub fn invert(self) -> Self {
        match self {
            Self::NodeTypeChanged { left, right } => Self::NodeTypeChanged {
                left: right,
                right: left,
            },
            Self::PropertyPresenceChanged {
                left,
                right,
                property,
                left_presence,
                right_presence,
            } => Self::PropertyPresenceChanged {
                left: right,
                right: left,
                property,
                left_presence: right_presence,
                right_presence: left_presence,
            },
            Self::ValueChanged {
                location,
                left,
                right,
            } => Self::ValueChanged {
                location: ValueLocation {
                    left: location.right,
                    right: location.left,
                    property: location.property,
                    left_index: location.right_index,
                    right_index: location.left_index,
                },
                left: right,
                right: left,
            },
            Self::ParentChanged {
                left,
                right,
                left_parent,
                right_parent,
                left_property,
                right_property,
            } => Self::ParentChanged {
                left: right,
                right: left,
                left_parent: right_parent,
                right_parent: left_parent,
                left_property: right_property,
                right_property: left_property,
            },
            Self::Reordered {
                left,
                right,
                left_scope,
                right_scope,
                property,
            } => Self::Reordered {
                left: right,
                right: left,
                left_scope: right_scope,
                right_scope: left_scope,
                property,
            },
        }
    }
}

impl PartialOrd for Difference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Difference {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

/// An alignment of two nodes, and the differences between their paired occurrences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    /// The version of the comparison format
    pub format_version: ComparisonFormatVersion,

    /// The algorithm, projection and policy that produced this comparison
    pub algorithm: AlgorithmInfo,

    /// The alignment the differences were derived from
    pub alignment: Alignment,

    /// The differences, in canonical order
    pub differences: Vec<Difference>,
}

impl Comparison {
    /// Create a comparison, putting its differences into canonical order
    pub fn new(alignment: Alignment, differences: Vec<Difference>) -> Self {
        let mut comparison = Self {
            format_version: ComparisonFormatVersion::V1,
            algorithm: alignment.algorithm.clone(),
            alignment,
            differences,
        };
        comparison.canonicalize();
        comparison
    }

    /// Put the differences into canonical order
    pub fn canonicalize(&mut self) {
        self.alignment.canonicalize();
        self.differences.sort();
    }

    /// Whether the two compared nodes are equal
    ///
    /// Because one-sided structure lives in the alignment rather than in the
    /// differences, the difference list alone is not an equality predicate: equality is
    /// no one-sided correspondences *and* no differences, which holds exactly when the
    /// two canonical projections are equal.
    pub fn is_equal(&self) -> bool {
        !self.alignment.has_one_sided() && self.differences.is_empty()
    }

    /// Invert this comparison, as though the two inputs had been swapped
    pub fn invert(self) -> Self {
        let mut inverted = Self {
            format_version: self.format_version,
            algorithm: self.algorithm,
            alignment: self.alignment.invert(),
            differences: self
                .differences
                .into_iter()
                .map(Difference::invert)
                .collect(),
        };
        inverted.canonicalize();
        inverted
    }
}
