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

use std::{cmp::Ordering, collections::HashSet};

use serde::{Deserialize, Serialize};

use stencila_node_path::NodePath;
use stencila_node_type::NodeProperty;

use crate::{
    alignment::{AlgorithmInfo, Alignment, Correspondence, NodeRef},
    error::{CompareError, CompareResult},
    filter::DifferenceFilter,
    scalar::ScalarValue,
};

/// The version of the comparison format
///
/// Versioned independently of the alignment format, because an alignment is also a
/// deliverable in its own right.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComparisonFormatVersion {
    /// The initial format
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
    /// rather than a single property list, and only ever paired with `Present` on the
    /// other side: a property that one type does not declare and the other does not
    /// carry says nothing about the two nodes, and the type change that made the
    /// property sets differ is already reported for the pair.
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
        self.sort_key()
            .cmp(&other.sort_key())
            .then_with(|| match (self, other) {
                (
                    Self::NodeTypeChanged { left, right },
                    Self::NodeTypeChanged {
                        left: other_left,
                        right: other_right,
                    },
                ) => (left, right).cmp(&(other_left, other_right)),
                (
                    Self::PropertyPresenceChanged {
                        left,
                        right,
                        property,
                        left_presence,
                        right_presence,
                    },
                    Self::PropertyPresenceChanged {
                        left: other_left,
                        right: other_right,
                        property: other_property,
                        left_presence: other_left_presence,
                        right_presence: other_right_presence,
                    },
                ) => (left, right, property, left_presence, right_presence).cmp(&(
                    other_left,
                    other_right,
                    other_property,
                    other_left_presence,
                    other_right_presence,
                )),
                (
                    Self::ValueChanged {
                        location,
                        left,
                        right,
                    },
                    Self::ValueChanged {
                        location: other_location,
                        left: other_left,
                        right: other_right,
                    },
                ) => (location, left, right).cmp(&(other_location, other_left, other_right)),
                (
                    Self::ParentChanged {
                        left,
                        right,
                        left_parent,
                        right_parent,
                        left_property,
                        right_property,
                    },
                    Self::ParentChanged {
                        left: other_left,
                        right: other_right,
                        left_parent: other_left_parent,
                        right_parent: other_right_parent,
                        left_property: other_left_property,
                        right_property: other_right_property,
                    },
                ) => (
                    left,
                    right,
                    left_parent,
                    right_parent,
                    left_property,
                    right_property,
                )
                    .cmp(&(
                        other_left,
                        other_right,
                        other_left_parent,
                        other_right_parent,
                        other_left_property,
                        other_right_property,
                    )),
                (
                    Self::Reordered {
                        left,
                        right,
                        left_scope,
                        right_scope,
                        property,
                    },
                    Self::Reordered {
                        left: other_left,
                        right: other_right,
                        left_scope: other_left_scope,
                        right_scope: other_right_scope,
                        property: other_property,
                    },
                ) => (left, right, left_scope, right_scope, property).cmp(&(
                    other_left,
                    other_right,
                    other_left_scope,
                    other_right_scope,
                    other_property,
                )),
                _ => Ordering::Equal,
            })
    }
}

/// An alignment of two nodes, and the differences between their paired occurrences
///
/// Deserialization restores canonical order and verifies that every difference refers
/// to a pair in the embedded alignment. Call [`Comparison::validate`] with the original
/// snapshots before trusting a deserialized artifact's complete coverage.
///
/// A comparison may be *filtered*, in which case it reports only the differences its
/// [`Comparison::filter`] selects. The embedded alignment is unaffected and stays
/// complete, so a filtered comparison is still a full account of what corresponds; only
/// the observations about paired occurrences are narrowed. The filter and the number of
/// differences it suppressed are both carried by the artifact, so a filtered comparison
/// can never be mistaken for an exhaustive one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "ComparisonData")]
pub struct Comparison {
    /// The version of the comparison format
    format_version: ComparisonFormatVersion,

    /// The alignment the differences were derived from
    alignment: Alignment,

    /// The differences, in canonical order
    differences: Vec<Difference>,

    /// The filter that selected which differences are reported
    #[serde(skip_serializing_if = "DifferenceFilter::is_empty")]
    filter: DifferenceFilter,

    /// How many derived differences the filter suppressed
    ///
    /// Stored rather than derived, because the suppressed differences themselves are
    /// not retained. One-sided suppression is not stored, because the alignment is
    /// complete and so it can always be recomputed from the filter.
    #[serde(skip_serializing_if = "is_zero")]
    suppressed_differences: usize,
}

/// How many one-sided correspondences a filter reports and suppresses, per side
///
/// Indexed by side: `[left, right]`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OneSidedTally {
    /// The correspondences the filter reports
    pub reported: [usize; 2],

    /// The correspondences the filter suppresses
    pub suppressed: [usize; 2],
}

impl OneSidedTally {
    /// The left-only and right-only correspondences that are reported
    pub fn left_only(&self) -> usize {
        self.reported[0]
    }

    /// The right-only correspondences that are reported
    pub fn right_only(&self) -> usize {
        self.reported[1]
    }

    /// How many one-sided correspondences the filter suppressed altogether
    pub fn suppressed_total(&self) -> usize {
        self.suppressed[0] + self.suppressed[1]
    }
}

/// Whether a count is zero, so that an unfiltered comparison serializes unchanged
fn is_zero(count: &usize) -> bool {
    *count == 0
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComparisonData {
    format_version: String,
    alignment: Alignment,
    differences: Vec<Difference>,
    #[serde(default)]
    filter: DifferenceFilter,
    #[serde(default)]
    suppressed_differences: usize,
}

impl TryFrom<ComparisonData> for Comparison {
    type Error = CompareError;

    fn try_from(data: ComparisonData) -> Result<Self, Self::Error> {
        let format_version = match data.format_version.as_str() {
            "1" => ComparisonFormatVersion::V1,
            version => {
                return Err(CompareError::UnsupportedVersion {
                    artifact: "comparison",
                    version: version.to_string(),
                });
            }
        };
        Self::new_with_version(
            format_version,
            data.alignment,
            data.differences,
            data.filter,
            data.suppressed_differences,
        )
    }
}

impl Comparison {
    /// Create an unfiltered comparison, putting its differences into canonical order
    pub(crate) fn new(alignment: Alignment, differences: Vec<Difference>) -> CompareResult<Self> {
        Self::new_with_version(
            ComparisonFormatVersion::V1,
            alignment,
            differences,
            DifferenceFilter::none(),
            0,
        )
    }

    /// Create a comparison reporting only the differences a filter selects
    ///
    /// The differences are filtered here, rather than while they are derived, so that
    /// derivation stays a pure function of the alignment and every difference is
    /// produced before any is hidden.
    pub(crate) fn new_filtered(
        alignment: Alignment,
        differences: Vec<Difference>,
        filter: DifferenceFilter,
    ) -> CompareResult<Self> {
        if filter.is_empty() {
            return Self::new(alignment, differences);
        }

        let derived = differences.len();
        let differences: Vec<Difference> = differences
            .into_iter()
            .filter(|difference| filter.allows_difference(difference))
            .collect();
        let suppressed = derived - differences.len();

        Self::new_with_version(
            ComparisonFormatVersion::V1,
            alignment,
            differences,
            filter,
            suppressed,
        )
    }

    fn new_with_version(
        format_version: ComparisonFormatVersion,
        alignment: Alignment,
        differences: Vec<Difference>,
        filter: DifferenceFilter,
        suppressed_differences: usize,
    ) -> CompareResult<Self> {
        let mut comparison = Self {
            format_version,
            alignment,
            differences,
            filter,
            suppressed_differences,
        };
        comparison.canonicalize();
        comparison.validate_local()?;
        Ok(comparison)
    }

    /// The version of the comparison format
    pub fn format_version(&self) -> ComparisonFormatVersion {
        self.format_version
    }

    /// The algorithm, projection and policy that produced this comparison
    pub fn algorithm(&self) -> &AlgorithmInfo {
        self.alignment.algorithm()
    }

    /// The alignment the differences were derived from
    pub fn alignment(&self) -> &Alignment {
        &self.alignment
    }

    /// The differences, in canonical order
    ///
    /// Only those the [`Comparison::filter`] selects, when the comparison is filtered.
    pub fn differences(&self) -> &[Difference] {
        &self.differences
    }

    /// The filter that selected which differences are reported
    ///
    /// Empty when the comparison reports every difference it derived.
    pub fn filter(&self) -> &DifferenceFilter {
        &self.filter
    }

    /// Whether this comparison reports only some of the differences it derived
    pub fn is_filtered(&self) -> bool {
        !self.filter.is_empty()
    }

    /// How many derived differences the filter suppressed
    pub fn suppressed_differences(&self) -> usize {
        self.suppressed_differences
    }

    /// How many one-sided correspondences the filter reports and suppresses
    ///
    /// Decided at the root of each one-sided subtree, not per occurrence. Every
    /// structured descendant of a one-sided occurrence has its own record, so testing
    /// each against the filter separately would hide an excluded `Link` while still
    /// reporting the text inside it. A subtree is reported, or hidden, whole.
    ///
    /// Recomputed from the complete alignment rather than stored, so that it stays
    /// correct for a deserialized artifact.
    pub fn one_sided_tally(&self) -> OneSidedTally {
        let mut tally = OneSidedTally::default();

        // Correspondences are in canonical order, so on each side the descendants of a
        // one-sided root immediately follow it and inherit its verdict
        let mut reporting = [true, true];

        for correspondence in self.alignment.correspondences() {
            let (side, node, ancestor) = match correspondence {
                Correspondence::Paired { .. } => continue,
                Correspondence::LeftOnly {
                    left,
                    nearest_one_sided_ancestor,
                    ..
                } => (0, left, nearest_one_sided_ancestor),
                Correspondence::RightOnly {
                    right,
                    nearest_one_sided_ancestor,
                    ..
                } => (1, right, nearest_one_sided_ancestor),
            };

            if ancestor.is_none() {
                reporting[side] = self.filter.allows_node(node);
            }

            let counts = if reporting[side] {
                &mut tally.reported
            } else {
                &mut tally.suppressed
            };
            counts[side] += 1;
        }

        tally
    }

    /// Whether any one-sided correspondence survives the filter
    fn has_reported_one_sided(&self) -> bool {
        let reported = self.one_sided_tally().reported;
        reported[0] + reported[1] > 0
    }

    /// Put the differences into canonical order
    fn canonicalize(&mut self) {
        self.differences.sort();
    }

    /// Validate this comparison against the two original snapshots
    pub fn validate(
        &self,
        left: &stencila_schema::Node,
        right: &stencila_schema::Node,
    ) -> CompareResult<()> {
        self.validate_local()?;
        self.alignment.validate(left, right)
    }

    fn validate_local(&self) -> CompareResult<()> {
        let pairs: HashSet<_> = self
            .alignment
            .pairs()
            .map(|(left, right, ..)| (left, right))
            .collect();

        for difference in &self.differences {
            if !pairs.contains(&(difference.left(), difference.right())) {
                return Err(CompareError::Invariant {
                    message: format!(
                        "A difference refers to an unpaired correspondence between `{}` and `{}`",
                        difference.left().path,
                        difference.right().path
                    ),
                });
            }
        }

        for duplicates in self.differences.windows(2) {
            if duplicates[0] == duplicates[1] {
                return Err(CompareError::Invariant {
                    message: "The same difference occurs more than once".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Whether the two compared nodes are equal, as far as this comparison reports
    ///
    /// Because one-sided structure lives in the alignment rather than in the
    /// differences, the difference list alone is not an equality predicate: equality is
    /// no one-sided correspondences *and* no differences.
    ///
    /// For an unfiltered comparison this holds exactly when the two canonical
    /// projections are equal. For a filtered one it is equality *modulo the filter*:
    /// the two nodes do not differ in any way the filter reports. That is what makes a
    /// filter usable as a round-trip gate, and why the artifact carries the filter that
    /// produced the verdict. Use [`Comparison::is_equal_unfiltered`] for the stricter
    /// question.
    pub fn is_equal(&self) -> bool {
        self.differences.is_empty() && !self.has_reported_one_sided()
    }

    /// Whether the two compared nodes are equal in every respect, filter or not
    ///
    /// Always answerable, even for a filtered comparison: the suppressed differences
    /// are not retained but their number is, and the embedded alignment is complete
    /// whatever the filter, so unfiltered equality is exactly no differences derived and
    /// no one-sided correspondences at all.
    pub fn is_equal_unfiltered(&self) -> bool {
        self.differences.is_empty()
            && self.suppressed_differences == 0
            && !self.alignment.has_one_sided()
    }

    /// Invert this comparison, as though the two inputs had been swapped
    pub fn invert(self) -> Self {
        let mut inverted = Self {
            format_version: self.format_version,
            alignment: self.alignment.invert(),
            differences: self
                .differences
                .into_iter()
                .map(Difference::invert)
                .collect(),
            // A selector matches either side of a pair, so a filter is side-symmetric
            // and inverts to itself
            filter: self.filter,
            suppressed_differences: self.suppressed_differences,
        };
        inverted.canonicalize();
        inverted
    }
}
