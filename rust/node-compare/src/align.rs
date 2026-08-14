//! Building an [`Alignment`] from two projections
//!
//! Correspondence is established locally, working down from the two roots through
//! already paired parents, so that ordinary insertions, deletions and local reordering
//! never trigger unrelated whole-document matches.
//!
//! Where the schema already determines correspondence, no scoring is involved:
//!
//! - the two caller-selected roots pair;
//! - a structured value in the same singular property of two paired parents pairs with
//!   its counterpart;
//! - an optional singular structured value present on only one side is one-sided.
//!
//! Singular pairs stay paired even when their types or contents differ. Treating a
//! complete replacement as a removal plus an addition, on the strength of a similarity
//! threshold, would throw away the strongest structural evidence available.

use std::collections::{HashMap, HashSet};

use stencila_node_path::NodePath;
use stencila_node_type::NodeProperty;

use crate::{
    alignment::{
        AlgorithmInfo, Alignment, AlignmentCost, AlignmentSignal, Correspondence, EvidenceValue,
        MatchEvidence, MatchInfo, MatchRule, NodeRef, UnmatchedReason,
    },
    error::{CompareError, CompareResult, Side},
    options::CompareOptions,
    policy::{ALGORITHM_NAME, ALGORITHM_VERSION, POLICY_NAME, gap_cost},
    projection::{Item, OccurrenceId, PROJECTION_VERSION, ProjectedProperty, Projection, Root},
};

/// Information about the algorithm that produced an artifact
pub(crate) fn algorithm_info() -> AlgorithmInfo {
    AlgorithmInfo {
        name: ALGORITHM_NAME.to_string(),
        version: ALGORITHM_VERSION.to_string(),
        projection_version: PROJECTION_VERSION.to_string(),
        policy: POLICY_NAME.to_string(),
    }
}

/// Aligns two projections
pub(crate) struct Aligner<'projection> {
    left: &'projection Projection,
    right: &'projection Projection,

    #[allow(dead_code)]
    options: &'projection CompareOptions,

    /// The correspondences established so far
    correspondences: Vec<Correspondence>,

    /// The left occurrences already covered, used to enforce complete, single coverage
    left_covered: HashSet<OccurrenceId>,

    /// The right occurrences already covered
    right_covered: HashSet<OccurrenceId>,
}

impl<'projection> Aligner<'projection> {
    /// Create an aligner for two projections
    pub fn new(
        left: &'projection Projection,
        right: &'projection Projection,
        options: &'projection CompareOptions,
    ) -> Self {
        Self {
            left,
            right,
            options,
            correspondences: Vec::new(),
            left_covered: HashSet::new(),
            right_covered: HashSet::new(),
        }
    }

    /// Align the two projections
    pub fn align(mut self) -> CompareResult<Alignment> {
        self.align_roots()?;
        self.check_coverage()?;

        Ok(Alignment::new(algorithm_info(), self.correspondences))
    }

    /// A reference to a structured occurrence
    fn node_ref(&self, projection: &Projection, id: OccurrenceId) -> CompareResult<NodeRef> {
        let occurrence = projection.occurrence(id)?;
        Ok(NodeRef::new(occurrence.path.clone(), occurrence.node_type))
    }

    /// A reference to the root of a projection
    ///
    /// The selected roots always receive a root correspondence, so a scalar root is
    /// the one place where a correspondence record refers to something other than a
    /// structured occurrence.
    fn root_ref(&self, projection: &Projection) -> CompareResult<NodeRef> {
        Ok(match projection.root() {
            Root::Structured(id) => self.node_ref(projection, *id)?,
            Root::Scalar(..) => NodeRef::new(NodePath::new(), projection.root_node_type()?),
        })
    }

    /// Pair the two caller-selected roots, then align downwards
    fn align_roots(&mut self) -> CompareResult<()> {
        let left_ref = self.root_ref(self.left)?;
        let right_ref = self.root_ref(self.right)?;

        let (left_root, right_root) = (self.left.root().clone(), self.right.root().clone());

        let left_gap_cost = match &left_root {
            Root::Structured(id) => gap_cost(self.left, *id)?,
            Root::Scalar(..) => AlignmentCost::ONE,
        };
        let right_gap_cost = match &right_root {
            Root::Structured(id) => gap_cost(self.right, *id)?,
            Root::Scalar(..) => AlignmentCost::ONE,
        };

        let same_type = left_ref.node_type == right_ref.node_type;
        self.correspondences.push(Correspondence::Paired {
            left: left_ref,
            right: right_ref,
            match_info: MatchInfo {
                rule: MatchRule::Root,
                // The roots are determined by the caller, so pairing them is free
                pair_cost: AlignmentCost::ZERO,
                left_gap_cost,
                right_gap_cost,
                evidence: vec![MatchEvidence {
                    signal: AlignmentSignal::NodeType,
                    value: EvidenceValue::Boolean { value: same_type },
                    contribution: AlignmentCost::ZERO,
                }],
            },
        });

        match (left_root, right_root) {
            (Root::Structured(left), Root::Structured(right)) => {
                self.left_covered.insert(left);
                self.right_covered.insert(right);
                self.align_properties(left, right)?;
            }
            (Root::Structured(left), Root::Scalar(..)) => {
                // The structured root is paired with the scalar root, but nothing
                // below it has a counterpart
                self.left_covered.insert(left);
                self.one_sided_descendants(Side::Left, left, None)?;
            }
            (Root::Scalar(..), Root::Structured(right)) => {
                self.right_covered.insert(right);
                self.one_sided_descendants(Side::Right, right, None)?;
            }
            (Root::Scalar(..), Root::Scalar(..)) => {}
        }

        Ok(())
    }

    /// Align the properties of two paired occurrences
    fn align_properties(&mut self, left: OccurrenceId, right: OccurrenceId) -> CompareResult<()> {
        let left_properties = &self.left.occurrence(left)?.properties;
        let right_properties = &self.right.occurrence(right)?.properties;

        let right_by_property: HashMap<NodeProperty, &ProjectedProperty> = right_properties
            .iter()
            .map(|property| (property.decl.property, property))
            .collect();
        let left_by_property: HashMap<NodeProperty, &ProjectedProperty> = left_properties
            .iter()
            .map(|property| (property.decl.property, property))
            .collect();

        // Properties declared by the left type, in their declared order, then those
        // declared only by the right type. A cross-type pair has a property union
        // rather than a single list, and this keeps the traversal deterministic.
        for left_property in left_properties {
            let property = left_property.decl.property;
            match right_by_property.get(&property) {
                Some(right_property) => self.align_property(left_property, right_property)?,
                None => self.one_sided_property(Side::Left, left_property)?,
            }
        }
        for right_property in right_properties {
            if !left_by_property.contains_key(&right_property.decl.property) {
                self.one_sided_property(Side::Right, right_property)?;
            }
        }

        Ok(())
    }

    /// Align one property that both sides declare
    fn align_property(
        &mut self,
        left: &ProjectedProperty,
        right: &ProjectedProperty,
    ) -> CompareResult<()> {
        // A structured value present in the same singular property of two paired
        // parents pairs with its counterpart, whatever their types or contents; a
        // singular value present on only one side is one-sided
        let repeated = left.decl.repeated || right.decl.repeated;
        let rule = if repeated {
            MatchRule::SequenceAlignment
        } else {
            MatchRule::SingularProperty
        };

        let left_items = structured_items(left);
        let right_items = structured_items(right);

        // Positional pairing. Ordered sequence alignment with explicit gaps replaces
        // this for repeated properties.
        let paired = left_items.len().min(right_items.len());
        for index in 0..paired {
            self.pair(left_items[index], right_items[index], rule)?;
        }
        for &id in &left_items[paired..] {
            self.one_sided(Side::Left, id, UnmatchedReason::NoCompatibleCandidate, None)?;
        }
        for &id in &right_items[paired..] {
            self.one_sided(
                Side::Right,
                id,
                UnmatchedReason::NoCompatibleCandidate,
                None,
            )?;
        }

        Ok(())
    }

    /// Record every structured item of a property that only one side has
    fn one_sided_property(
        &mut self,
        side: Side,
        property: &ProjectedProperty,
    ) -> CompareResult<()> {
        for id in structured_items(property) {
            self.one_sided(side, id, UnmatchedReason::NoCompatibleCandidate, None)?;
        }

        Ok(())
    }

    /// Pair two occurrences, then align their properties
    fn pair(
        &mut self,
        left: OccurrenceId,
        right: OccurrenceId,
        rule: MatchRule,
    ) -> CompareResult<()> {
        if !self.left_covered.insert(left) {
            return Err(CompareError::Invariant {
                message: format!(
                    "The left occurrence at `{path}` is covered more than once",
                    path = self.left.occurrence(left)?.path
                ),
            });
        }
        if !self.right_covered.insert(right) {
            return Err(CompareError::Invariant {
                message: format!(
                    "The right occurrence at `{path}` is covered more than once",
                    path = self.right.occurrence(right)?.path
                ),
            });
        }

        let left_ref = self.node_ref(self.left, left)?;
        let right_ref = self.node_ref(self.right, right)?;
        let same_type = left_ref.node_type == right_ref.node_type;

        self.correspondences.push(Correspondence::Paired {
            left: left_ref,
            right: right_ref,
            match_info: MatchInfo {
                rule,
                pair_cost: AlignmentCost::ZERO,
                left_gap_cost: gap_cost(self.left, left)?,
                right_gap_cost: gap_cost(self.right, right)?,
                evidence: vec![MatchEvidence {
                    signal: AlignmentSignal::NodeType,
                    value: EvidenceValue::Boolean { value: same_type },
                    contribution: AlignmentCost::ZERO,
                }],
            },
        });

        self.align_properties(left, right)
    }

    /// Record a one-sided occurrence and every structured occurrence below it
    ///
    /// A one-sided subtree emits a record for every structured descendant, not only
    /// the subtree root, so that a consumer can count any structured descendants of
    /// interest. Each record names its nearest one-sided ancestor, so that a
    /// presentation layer can collapse the subtree again.
    fn one_sided(
        &mut self,
        side: Side,
        id: OccurrenceId,
        reason: UnmatchedReason,
        ancestor: Option<NodeRef>,
    ) -> CompareResult<()> {
        let projection = self.projection(side);
        let node_ref = self.node_ref(projection, id)?;

        let covered = match side {
            Side::Left => self.left_covered.insert(id),
            Side::Right => self.right_covered.insert(id),
        };
        if !covered {
            return Err(CompareError::Invariant {
                message: format!(
                    "The {side} occurrence at `{path}` is covered more than once",
                    path = node_ref.path
                ),
            });
        }

        self.correspondences.push(match side {
            Side::Left => Correspondence::LeftOnly {
                left: node_ref.clone(),
                reason,
                nearest_one_sided_ancestor: ancestor,
            },
            Side::Right => Correspondence::RightOnly {
                right: node_ref.clone(),
                reason,
                nearest_one_sided_ancestor: ancestor,
            },
        });

        self.one_sided_descendants(side, id, Some(node_ref))
    }

    /// Record the structured descendants of an occurrence as one-sided
    fn one_sided_descendants(
        &mut self,
        side: Side,
        id: OccurrenceId,
        ancestor: Option<NodeRef>,
    ) -> CompareResult<()> {
        let children = crate::policy::children(self.projection(side), id)?;
        for child in children {
            self.one_sided(
                side,
                child,
                UnmatchedReason::NoCompatibleCandidate,
                ancestor.clone(),
            )?;
        }

        Ok(())
    }

    /// The projection for a side
    fn projection(&self, side: Side) -> &'projection Projection {
        match side {
            Side::Left => self.left,
            Side::Right => self.right,
        }
    }

    /// Check that every occurrence on both sides is covered exactly once
    ///
    /// Single coverage is enforced as records are added; this checks that nothing was
    /// left out. A violation is a bug, and returns an error rather than a partial
    /// artifact.
    fn check_coverage(&self) -> CompareResult<()> {
        for (side, projection, covered) in [
            (Side::Left, self.left, &self.left_covered),
            (Side::Right, self.right, &self.right_covered),
        ] {
            for occurrence in projection.occurrences() {
                if !covered.contains(&occurrence.id) {
                    return Err(CompareError::Invariant {
                        message: format!(
                            "The {side} occurrence at `{path}` is not covered by the alignment",
                            path = occurrence.path
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

/// The structured items of a property, in order
fn structured_items(property: &ProjectedProperty) -> Vec<OccurrenceId> {
    property
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Structured(id) => Some(*id),
            Item::Scalar(..) => None,
        })
        .collect()
}
