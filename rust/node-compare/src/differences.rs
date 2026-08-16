//! Deriving differences from a completed alignment
//!
//! Differences are derived only after the final alignment is complete, so that a
//! difference is always a statement about a pair that the alignment actually selected.
//!
//! For each paired occurrence: compare the node type; enumerate the compatible
//! property union of the two projected types; compare property declaration and
//! presence; and compare scalar leaves. Structured properties recurse through the
//! alignment rather than being recorded as part of their containing value, and the
//! completed collection alignment is used for repeated mixed values.
//!
//! One-sided occurrences produce no differences at all. Their exhaustive one-sided
//! correspondence records already capture the absent structure, and emitting leaf
//! values for them as well would represent every missing subtree repeatedly at its
//! root, at its descendants and at its leaves.

use std::collections::HashMap;

use stencila_node_type::NodeProperty;
use stencila_schema::ValueKind;

use crate::{
    align::Aligned,
    alignment::NodeRef,
    comparison::{Difference, PropertyPresence, ValueLocation, ValueState},
    error::CompareResult,
    projection::{
        Item, OccurrenceId, Presence, ProjectedProperty, Projection, PropertyPair, Root,
        property_union,
    },
    scalar::ScalarValue,
    sequence::Step,
};

/// Derive the differences between two aligned projections
pub(crate) fn derive(
    left: &Projection,
    right: &Projection,
    aligned: &Aligned,
) -> CompareResult<Vec<Difference>> {
    let mut differ = Differ {
        left,
        right,
        pairing: aligned.pairs.iter().copied().collect(),
        property_alignments: aligned
            .properties
            .iter()
            .map(|property| {
                (
                    (
                        property.left_parent,
                        property.right_parent,
                        property.property,
                    ),
                    property.steps.as_slice(),
                )
            })
            .collect(),
        differences: Vec::new(),
    };

    differ.derive_roots()?;
    for (left, right) in &aligned.pairs {
        differ.derive_pair(*left, *right)?;
        differ.derive_parent(*left, *right)?;
    }

    Ok(differ.differences)
}

struct Differ<'projection> {
    left: &'projection Projection,
    right: &'projection Projection,

    /// The counterpart of each left occurrence
    pairing: HashMap<OccurrenceId, OccurrenceId>,

    /// The completed collection alignment of each repeated property of each pair
    property_alignments: HashMap<(OccurrenceId, OccurrenceId, NodeProperty), &'projection [Step]>,

    differences: Vec<Difference>,
}

impl Differ<'_> {
    /// A reference to a structured occurrence
    fn node_ref(&self, projection: &Projection, id: OccurrenceId) -> CompareResult<NodeRef> {
        Ok(projection.occurrence(id)?.node_ref())
    }

    /// A reference to the root of a projection
    fn root_ref(&self, projection: &Projection) -> CompareResult<NodeRef> {
        projection.root_ref()
    }

    /// Compare two roots that are not both structured
    ///
    /// Two same-type primitive roots compare through a root value location. A
    /// primitive-versus-structured root pair, or an incompatible primitive pair,
    /// records a node type change, without forcing their contents into a structural
    /// comparison.
    fn derive_roots(&mut self) -> CompareResult<()> {
        let (left_root, right_root) = (self.left.root(), self.right.root());
        let (Root::Scalar(left_value), Root::Scalar(right_value)) = (left_root, right_root) else {
            // A scalar root paired with a structured root differs in node type; two
            // structured roots are an ordinary pair, handled with every other pair
            if matches!(left_root, Root::Structured(..))
                == matches!(right_root, Root::Structured(..))
            {
                return Ok(());
            }
            self.differences.push(Difference::NodeTypeChanged {
                left: self.root_ref(self.left)?,
                right: self.root_ref(self.right)?,
            });
            return Ok(());
        };

        let left_ref = self.root_ref(self.left)?;
        let right_ref = self.root_ref(self.right)?;

        if left_ref.node_type != right_ref.node_type {
            self.differences.push(Difference::NodeTypeChanged {
                left: left_ref,
                right: right_ref,
            });
            return Ok(());
        }

        if left_value != right_value {
            self.differences.push(Difference::ValueChanged {
                location: ValueLocation {
                    left: left_ref,
                    right: right_ref,
                    property: None,
                    left_index: None,
                    right_index: None,
                },
                left: ValueState::One {
                    value: left_value.clone(),
                },
                right: ValueState::One {
                    value: right_value.clone(),
                },
            });
        }

        Ok(())
    }

    /// Report a pair whose aligned parents, or containing properties, differ
    ///
    /// Movement is never inferred from raw index inequality: inserting one early
    /// sibling shifts every later sibling's index without moving any of them, and only
    /// a change of *aligned* parent or of containing property is a move.
    fn derive_parent(&mut self, left: OccurrenceId, right: OccurrenceId) -> CompareResult<()> {
        let left_occurrence = self.left.occurrence(left)?;
        let right_occurrence = self.right.occurrence(right)?;

        let same_parent = match (left_occurrence.parent, right_occurrence.parent) {
            (None, None) => true,
            (Some(left_parent), Some(right_parent)) => {
                self.pairing.get(&left_parent) == Some(&right_parent)
            }
            _ => false,
        };
        let same_property = left_occurrence.parent_property == right_occurrence.parent_property;

        if same_parent && same_property {
            return Ok(());
        }

        let parent_ref = |projection: &Projection, parent: Option<OccurrenceId>| {
            parent
                .map(|parent| self.node_ref(projection, parent))
                .transpose()
        };

        self.differences.push(Difference::ParentChanged {
            left: self.node_ref(self.left, left)?,
            right: self.node_ref(self.right, right)?,
            left_parent: parent_ref(self.left, left_occurrence.parent)?,
            right_parent: parent_ref(self.right, right_occurrence.parent)?,
            left_property: left_occurrence.parent_property,
            right_property: right_occurrence.parent_property,
        });

        Ok(())
    }

    /// Derive the differences of one paired occurrence
    fn derive_pair(&mut self, left: OccurrenceId, right: OccurrenceId) -> CompareResult<()> {
        let left_ref = self.node_ref(self.left, left)?;
        let right_ref = self.node_ref(self.right, right)?;

        if left_ref.node_type != right_ref.node_type {
            self.differences.push(Difference::NodeTypeChanged {
                left: left_ref.clone(),
                right: right_ref.clone(),
            });
        }

        let left_properties = &self.left.occurrence(left)?.properties;
        let right_properties = &self.right.occurrence(right)?.properties;

        for properties in property_union(left_properties, right_properties) {
            match properties {
                PropertyPair::Both(left_property, right_property) => self.derive_property(
                    (left, &left_ref, left_property),
                    (right, &right_ref, right_property),
                )?,
                PropertyPair::LeftOnly(property) => {
                    self.differences.push(Difference::PropertyPresenceChanged {
                        left: left_ref.clone(),
                        right: right_ref.clone(),
                        property: property.decl.property,
                        left_presence: presence(property),
                        right_presence: PropertyPresence::Undeclared,
                    })
                }
                PropertyPair::RightOnly(property) => {
                    self.differences.push(Difference::PropertyPresenceChanged {
                        left: left_ref.clone(),
                        right: right_ref.clone(),
                        property: property.decl.property,
                        left_presence: PropertyPresence::Undeclared,
                        right_presence: presence(property),
                    })
                }
            }
        }

        Ok(())
    }

    /// Derive the differences of one property that both types declare
    fn derive_property(
        &mut self,
        left: (OccurrenceId, &NodeRef, &ProjectedProperty),
        right: (OccurrenceId, &NodeRef, &ProjectedProperty),
    ) -> CompareResult<()> {
        let (left_parent, left_ref, left_property) = left;
        let (right_parent, right_ref, right_property) = right;

        let location = |left_index, right_index| ValueLocation {
            left: left_ref.clone(),
            right: right_ref.clone(),
            property: Some(left_property.decl.property),
            left_index,
            right_index,
        };

        if left_property.presence != right_property.presence
            && left_property.decl.kind == ValueKind::Scalar
            && right_property.decl.kind == ValueKind::Scalar
        {
            self.differences.push(Difference::ValueChanged {
                location: location(None, None),
                left: scalar_property_state(left_property),
                right: scalar_property_state(right_property),
            });
            return Ok(());
        }

        if left_property.presence != right_property.presence {
            self.differences.push(Difference::PropertyPresenceChanged {
                left: left_ref.clone(),
                right: right_ref.clone(),
                property: left_property.decl.property,
                left_presence: presence(left_property),
                right_presence: presence(right_property),
            });
            return Ok(());
        }
        if left_property.presence == Presence::Absent {
            return Ok(());
        }

        // A singular property holds at most one value, so it is compared directly:
        // structured values are the alignment's business, and only a scalar on one
        // side needs recording
        if !left_property.decl.repeated && !right_property.decl.repeated {
            let left_value = scalar(left_property.items.first());
            let right_value = scalar(right_property.items.first());
            if left_value != right_value {
                self.differences.push(Difference::ValueChanged {
                    location: location(None, None),
                    left: state(left_value),
                    right: state(right_value),
                });
            }
            return Ok(());
        }

        // A homogeneous repeated scalar property is one atomic difference carrying
        // both complete typed sequences, rather than one difference per item
        if left_property.decl.kind == ValueKind::Scalar
            && right_property.decl.kind == ValueKind::Scalar
        {
            let left_values = scalars(left_property);
            let right_values = scalars(right_property);
            if left_values != right_values {
                self.differences.push(Difference::ValueChanged {
                    location: location(None, None),
                    left: ValueState::Many {
                        values: left_values,
                    },
                    right: ValueState::Many {
                        values: right_values,
                    },
                });
            }
            return Ok(());
        }

        // A mixed collection holds both typed objects and scalars. Its structured
        // items are correspondence records; its scalar items are indexed value
        // observations, because a scalar never becomes a fake node occurrence.
        let Some(steps) =
            self.property_alignments
                .get(&(left_parent, right_parent, left_property.decl.property))
        else {
            return Ok(());
        };

        for step in *steps {
            match *step {
                Step::Pair {
                    left: left_index,
                    right: right_index,
                } => {
                    let left_value = scalar(left_property.items.get(left_index));
                    let right_value = scalar(right_property.items.get(right_index));
                    if (left_value.is_some() || right_value.is_some()) && left_value != right_value
                    {
                        self.differences.push(Difference::ValueChanged {
                            location: location(Some(left_index), Some(right_index)),
                            left: state(left_value),
                            right: state(right_value),
                        });
                    }
                }
                Step::LeftGap { left: left_index } => {
                    if let Some(value) = scalar(left_property.items.get(left_index)) {
                        self.differences.push(Difference::ValueChanged {
                            location: location(Some(left_index), None),
                            left: ValueState::One {
                                value: value.clone(),
                            },
                            right: ValueState::Absent,
                        });
                    }
                }
                Step::RightGap { right: right_index } => {
                    if let Some(value) = scalar(right_property.items.get(right_index)) {
                        self.differences.push(Difference::ValueChanged {
                            location: location(None, Some(right_index)),
                            left: ValueState::Absent,
                            right: ValueState::One {
                                value: value.clone(),
                            },
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

/// The presence of a property, as reported in a difference
fn presence(property: &ProjectedProperty) -> PropertyPresence {
    match property.presence {
        Presence::Absent => PropertyPresence::Absent,
        Presence::Present => PropertyPresence::Present,
    }
}

/// The scalar value of an item, if it is one
fn scalar(item: Option<&Item>) -> Option<&ScalarValue> {
    match item {
        Some(Item::Scalar(value)) => Some(value),
        _ => None,
    }
}

/// The values of a property, when every one of its items is a scalar
fn scalars(property: &ProjectedProperty) -> Vec<ScalarValue> {
    property
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Scalar(value) => Some(value.clone()),
            Item::Structured(..) => None,
        })
        .collect()
}

/// The value state of a schema-declared scalar property
fn scalar_property_state(property: &ProjectedProperty) -> ValueState {
    if property.presence == Presence::Absent {
        ValueState::Absent
    } else if property.decl.repeated {
        ValueState::Many {
            values: scalars(property),
        }
    } else {
        state(scalar(property.items.first()))
    }
}

/// A value state from an optional value
fn state(value: Option<&ScalarValue>) -> ValueState {
    match value {
        Some(value) => ValueState::One {
            value: value.clone(),
        },
        None => ValueState::Absent,
    }
}
