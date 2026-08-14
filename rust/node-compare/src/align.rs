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
//!
//! Repeated properties do require candidate matching. They are aligned by fixing
//! verified unique anchors, partitioning the two sequences at those anchors, and
//! running an order-preserving dynamic program with explicit gaps within each
//! partition.

use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use stencila_node_path::NodePath;
use stencila_node_type::NodeProperty;

use crate::{
    alignment::{
        AlgorithmInfo, Alignment, AlignmentCost, AlignmentSignal, Correspondence, EvidenceValue,
        MatchEvidence, MatchInfo, MatchRule, NodeRef, UnmatchedReason,
    },
    anchors,
    error::{CompareError, CompareResult, Side},
    features::FeatureSet,
    fingerprint,
    options::CompareOptions,
    policy::{
        ALGORITHM_NAME, ALGORITHM_VERSION, CandidateKind, POLICY_NAME, gap_cost, item_gap_cost,
        pair_cost,
    },
    projection::{Item, OccurrenceId, PROJECTION_VERSION, ProjectedProperty, Projection, Root},
    sequence::{self, Costs, Step, TieKey},
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

/// How the items of one repeated property of one paired parent were aligned
///
/// Retained so that difference derivation can use the completed collection alignment
/// rather than re-deriving it, and so that the scalar items of a mixed collection,
/// which never become correspondence records, are still accounted for.
#[derive(Debug, Clone)]
#[allow(dead_code, reason = "read by difference derivation")]
pub(crate) struct PropertyAlignment {
    pub left_parent: OccurrenceId,
    pub right_parent: OccurrenceId,
    pub property: NodeProperty,
    pub steps: Vec<Step>,
}

/// Everything the aligner established
pub(crate) struct Aligned {
    pub alignment: Alignment,

    /// The paired occurrences, by projection id
    ///
    /// The same pairs as the alignment's paired correspondences, but by id rather than
    /// by path, so that difference derivation can reach the projected values without
    /// resolving paths.
    pub pairs: Vec<(OccurrenceId, OccurrenceId)>,

    /// How each repeated property of each paired parent was aligned
    pub properties: Vec<PropertyAlignment>,
}

/// Aligns two projections
pub(crate) struct Aligner<'projection> {
    left: &'projection Projection,
    left_features: &'projection FeatureSet,

    right: &'projection Projection,
    right_features: &'projection FeatureSet,

    options: &'projection CompareOptions,

    /// The correspondences established so far
    correspondences: Vec<Correspondence>,

    /// The paired occurrences, by projection id
    pairs: Vec<(OccurrenceId, OccurrenceId)>,

    /// How each repeated property of each paired parent was aligned
    properties: Vec<PropertyAlignment>,

    /// The left occurrences already covered, used to enforce complete, single coverage
    left_covered: HashSet<OccurrenceId>,

    /// The right occurrences already covered
    right_covered: HashSet<OccurrenceId>,

    /// The candidate cells consumed by sequence alignment so far
    cells_used: usize,
}

impl<'projection> Aligner<'projection> {
    /// Create an aligner for two projections
    pub fn new(
        left: &'projection Projection,
        left_features: &'projection FeatureSet,
        right: &'projection Projection,
        right_features: &'projection FeatureSet,
        options: &'projection CompareOptions,
    ) -> Self {
        Self {
            left,
            left_features,
            right,
            right_features,
            options,
            correspondences: Vec::new(),
            pairs: Vec::new(),
            properties: Vec::new(),
            left_covered: HashSet::new(),
            right_covered: HashSet::new(),
            cells_used: 0,
        }
    }

    /// Align the two projections
    pub fn align(mut self) -> CompareResult<Aligned> {
        self.align_roots()?;
        self.check_coverage()?;

        Ok(Aligned {
            alignment: Alignment::new(algorithm_info(), self.correspondences),
            pairs: self.pairs,
            properties: self.properties,
        })
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
                self.pairs.push((left, right));
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
                Some(right_property) => {
                    self.align_property(left, right, left_property, right_property)?
                }
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
        left_parent: OccurrenceId,
        right_parent: OccurrenceId,
        left: &'projection ProjectedProperty,
        right: &'projection ProjectedProperty,
    ) -> CompareResult<()> {
        // A structured value present in the same singular property of two paired
        // parents pairs with its counterpart, whatever their types or contents; a
        // singular value present on only one side is one-sided. No scoring is needed,
        // because the schema has already determined the correspondence.
        if !left.decl.repeated && !right.decl.repeated {
            let left_items = structured_items(left);
            let right_items = structured_items(right);
            match (left_items.first(), right_items.first()) {
                (Some(left), Some(right)) => {
                    self.pair(*left, *right, MatchRule::SingularProperty, Vec::new())?
                }
                (Some(left), None) => self.one_sided(
                    Side::Left,
                    *left,
                    UnmatchedReason::NoCompatibleCandidate,
                    None,
                )?,
                (None, Some(right)) => self.one_sided(
                    Side::Right,
                    *right,
                    UnmatchedReason::NoCompatibleCandidate,
                    None,
                )?,
                (None, None) => {}
            }
            return Ok(());
        }

        self.align_sequence(left_parent, right_parent, left, right)
    }

    /// Align the items of a repeated property
    fn align_sequence(
        &mut self,
        left_parent: OccurrenceId,
        right_parent: OccurrenceId,
        left: &'projection ProjectedProperty,
        right: &'projection ProjectedProperty,
    ) -> CompareResult<()> {
        let (left_projection, right_projection) = (self.left, self.right);
        let (left_decl, right_decl) = (&left.decl, &right.decl);

        // A homogeneous scalar collection remains one sequence-valued property
        // difference rather than gaining item correspondence records, so there is
        // nothing for sequence alignment to do, and nothing to charge for it
        if all_scalar(left) && all_scalar(right) {
            return Ok(());
        }

        let left_kinds = candidate_kinds(left, self.left_features)?;
        let right_kinds = candidate_kinds(right, self.right_features)?;

        if left_kinds.is_empty() && right_kinds.is_empty() {
            return Ok(());
        }

        let compatible = |left_index: usize, right_index: usize| -> CompareResult<bool> {
            Ok(crate::policy::compatible(
                left_kinds[left_index],
                right_kinds[right_index],
                left_decl,
                right_decl,
            ))
        };
        let verified_eq = |left_index: usize, right_index: usize| -> CompareResult<bool> {
            match (&left.items[left_index], &right.items[right_index]) {
                (Item::Structured(left), Item::Structured(right)) => {
                    left_projection.eq_subtrees(*left, right_projection, *right)
                }
                (Item::Scalar(left), Item::Scalar(right)) => Ok(left == right),
                _ => Ok(false),
            }
        };

        // Verified unique anchors fix part of the alignment and partition the rest into
        // smaller gaps, which is what keeps the dynamic program affordable
        let anchors = anchors::find(
            &anchor_candidates(&left_kinds),
            &anchor_candidates(&right_kinds),
            &compatible,
            &verified_eq,
        )?;

        let segments = segments(&anchors, left_kinds.len(), right_kinds.len());
        self.charge_cells(&segments, left, right)?;

        let costs = Costs {
            pair: &|left_index, right_index| {
                Ok(pair_cost(
                    left_kinds[left_index],
                    right_kinds[right_index],
                    left_decl,
                    right_decl,
                )
                .cost)
            },
            left_gap: &|index| Ok(item_gap_cost(left_kinds[index])),
            right_gap: &|index| Ok(item_gap_cost(right_kinds[index])),
            left_key: &|index| Ok(tie_key(left_kinds[index], index)),
            right_key: &|index| Ok(tie_key(right_kinds[index], index)),
        };

        // The aligned segments between the anchors, and the anchored pairs themselves,
        // in sequence order
        let mut steps = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            steps.extend(sequence::align(
                segment.0.clone(),
                segment.1.clone(),
                &costs,
            )?);
            if let Some(anchor) = anchors.get(index) {
                steps.push(Step::Pair {
                    left: anchor.left,
                    right: anchor.right,
                });
            }
        }

        // A one-sided item either had no compatible candidate at all, or had some but
        // the gap cost less than every one of them
        let left_candidates = has_candidate(&left_kinds, &right_kinds, &compatible, Side::Left)?;
        let right_candidates = has_candidate(&right_kinds, &left_kinds, &compatible, Side::Right)?;

        let rules: HashMap<(usize, usize), MatchRule> = anchors
            .iter()
            .map(|anchor| ((anchor.left, anchor.right), anchor.rule))
            .collect();

        for step in &steps {
            match *step {
                Step::Pair {
                    left: left_index,
                    right: right_index,
                } => {
                    let (Item::Structured(left_id), Item::Structured(right_id)) =
                        (&left.items[left_index], &right.items[right_index])
                    else {
                        // Scalar items of a mixed collection are values, not
                        // occurrences: they produce indexed value observations rather
                        // than correspondence records
                        continue;
                    };
                    let rule = rules
                        .get(&(left_index, right_index))
                        .copied()
                        .unwrap_or(MatchRule::SequenceAlignment);
                    let candidate = pair_cost(
                        left_kinds[left_index],
                        right_kinds[right_index],
                        left_decl,
                        right_decl,
                    );
                    self.pair(*left_id, *right_id, rule, candidate.evidence)?;
                }
                Step::LeftGap { left: index } => {
                    if let Item::Structured(id) = &left.items[index] {
                        self.one_sided(
                            Side::Left,
                            *id,
                            unmatched_reason(left_candidates[index]),
                            None,
                        )?;
                    }
                }
                Step::RightGap { right: index } => {
                    if let Item::Structured(id) = &right.items[index] {
                        self.one_sided(
                            Side::Right,
                            *id,
                            unmatched_reason(right_candidates[index]),
                            None,
                        )?;
                    }
                }
            }
        }

        self.properties.push(PropertyAlignment {
            left_parent,
            right_parent,
            property: left_decl.property,
            steps,
        });

        Ok(())
    }

    /// Charge the candidate cells that aligning a property will use
    ///
    /// The budget is operational rather than semantic: exceeding it returns an error
    /// naming the two property paths and the required and allowed cells, never a
    /// silently approximate result and never a misleadingly large one-sided alignment.
    fn charge_cells(
        &mut self,
        segments: &[(Range<usize>, Range<usize>)],
        left: &ProjectedProperty,
        right: &ProjectedProperty,
    ) -> CompareResult<()> {
        let required: usize = segments
            .iter()
            .map(|(left, right)| sequence::cells(left.len(), right.len()))
            .sum();

        self.cells_used = self.cells_used.saturating_add(required);
        if self.cells_used > self.options.alignment_cell_budget {
            return Err(CompareError::BudgetExhausted {
                left_path: NodePath::from(left.decl.property),
                right_path: NodePath::from(right.decl.property),
                required: self.cells_used,
                allowed: self.options.alignment_cell_budget,
            });
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
        evidence: Vec<MatchEvidence>,
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

        // A pair the schema determined has no candidate scoring behind it, so its only
        // evidence is whether the two types agree
        let evidence = if evidence.is_empty() {
            vec![MatchEvidence {
                signal: AlignmentSignal::NodeType,
                value: EvidenceValue::Boolean { value: same_type },
                contribution: AlignmentCost::ZERO,
            }]
        } else {
            evidence
        };
        let pair_cost = evidence
            .iter()
            .fold(AlignmentCost::ZERO, |total, evidence| {
                total.saturating_add(evidence.contribution)
            });

        self.pairs.push((left, right));
        self.correspondences.push(Correspondence::Paired {
            left: left_ref,
            right: right_ref,
            match_info: MatchInfo {
                rule,
                pair_cost,
                left_gap_cost: gap_cost(self.left, left)?,
                right_gap_cost: gap_cost(self.right, right)?,
                evidence,
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

/// Whether every item of a property is a scalar
fn all_scalar(property: &ProjectedProperty) -> bool {
    property
        .items
        .iter()
        .all(|item| matches!(item, Item::Scalar(..)))
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

/// The candidate view of each item of a property
///
/// Sequence alignment operates over structured *and* scalar tokens, so that an exact
/// scalar item of a mixed union collection can anchor the structure around it.
fn candidate_kinds<'projection>(
    property: &'projection ProjectedProperty,
    features: &'projection FeatureSet,
) -> CompareResult<Vec<CandidateKind<'projection>>> {
    property
        .items
        .iter()
        .map(|item| {
            Ok(match item {
                Item::Structured(id) => CandidateKind::Structured(features.get(*id)?),
                Item::Scalar(value) => CandidateKind::Scalar(value),
            })
        })
        .collect()
}

/// The anchor view of each candidate
fn anchor_candidates(kinds: &[CandidateKind<'_>]) -> Vec<Option<anchors::Candidate>> {
    kinds
        .iter()
        .map(|kind| {
            Some(match kind {
                CandidateKind::Structured(features) => anchors::Candidate {
                    explicit_id: features.explicit_id.clone(),
                    fingerprint: features.fingerprint,
                    node_type: Some(features.node_type),
                },
                CandidateKind::Scalar(value) => anchors::Candidate {
                    explicit_id: None,
                    fingerprint: fingerprint::scalar(value),
                    node_type: None,
                },
            })
        })
        .collect()
}

/// The tie-break key of a candidate at a position
fn tie_key(kind: CandidateKind<'_>, position: usize) -> TieKey {
    match kind {
        CandidateKind::Structured(features) => TieKey {
            node_type: Some(features.node_type),
            fingerprint: features.fingerprint,
            position,
        },
        CandidateKind::Scalar(value) => TieKey {
            node_type: None,
            fingerprint: fingerprint::scalar(value),
            position,
        },
    }
}

/// The ranges between consecutive anchors, plus the range after the last one
fn segments(
    anchors: &[anchors::Anchor],
    left: usize,
    right: usize,
) -> Vec<(Range<usize>, Range<usize>)> {
    let mut segments = Vec::with_capacity(anchors.len() + 1);
    let (mut left_start, mut right_start) = (0usize, 0usize);

    for anchor in anchors {
        segments.push((left_start..anchor.left, right_start..anchor.right));
        left_start = anchor.left + 1;
        right_start = anchor.right + 1;
    }
    segments.push((left_start..left, right_start..right));

    segments
}

/// Whether each item of a sequence had any compatible candidate on the other side
fn has_candidate(
    kinds: &[CandidateKind<'_>],
    others: &[CandidateKind<'_>],
    compatible: &dyn Fn(usize, usize) -> CompareResult<bool>,
    side: Side,
) -> CompareResult<Vec<bool>> {
    let mut any = Vec::with_capacity(kinds.len());
    for index in 0..kinds.len() {
        let mut found = false;
        for other in 0..others.len() {
            let compatible = match side {
                Side::Left => compatible(index, other)?,
                Side::Right => compatible(other, index)?,
            };
            if compatible {
                found = true;
                break;
            }
        }
        any.push(found);
    }

    Ok(any)
}

/// Why an item that a sequence alignment left as a gap has no counterpart
fn unmatched_reason(had_candidate: bool) -> UnmatchedReason {
    if had_candidate {
        UnmatchedReason::GapCheaperThanPair
    } else {
        UnmatchedReason::NoCompatibleCandidate
    }
}
