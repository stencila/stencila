//! The canonical projection
//!
//! Both inputs are projected once into a uniform internal tree of typed occurrences,
//! property edges, presence, and scalar values, so that the alignment and difference
//! algorithms operate on one representation rather than on hundreds of generated Rust
//! types.
//!
//! The projection is built from the schema introspection seam
//! ([`stencila_schema::InspectNode`]), never from serialized JSON, because
//! serialization loses schema type, union, cardinality and presence information.
//!
//! # Intrinsic representation rules
//!
//! These are built in, not configurable:
//!
//! - `uid` is ignored, and the `type` discriminator is represented structurally as the
//!   [`NodeType`] of an occurrence rather than as a duplicate scalar property (both
//!   because the introspection seam does not report them as properties);
//! - a `Cord` projects to its string only, and its authorship is ignored entirely,
//!   matching the custom `PartialEq` implementation for `Cord`;
//! - dynamic object entries are canonically ordered by key, so insertion order affects
//!   neither comparison nor serialization.
//!
//! Everything else the schema declares — explicit `id`, provenance, authorship
//! outside a `Cord`, execution state, compilation messages — is projected and compared
//! by default.
//!
//! This module is an implementation detail of comparison rather than a stable artifact
//! contract. Its observable semantics are exercised through the crate's comparison
//! interface and its representation rules are covered by internal unit tests.

use std::collections::HashMap;

use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{InspectNode, InspectValue, PropertyDecl, ScalarRef};

use crate::{
    alignment::NodeRef,
    error::{CompareError, CompareResult, Side},
    fingerprint::Identity,
    scalar::ScalarValue,
};

/// One member of the deterministic property union of two occurrences
pub(crate) enum PropertyPair<'projection> {
    Both(
        &'projection ProjectedProperty,
        &'projection ProjectedProperty,
    ),
    LeftOnly(&'projection ProjectedProperty),
    RightOnly(&'projection ProjectedProperty),
}

/// The properties declared by the left occurrence in declaration order, followed by
/// properties declared only by the right occurrence
pub(crate) fn property_union<'projection>(
    left: &'projection [ProjectedProperty],
    right: &'projection [ProjectedProperty],
) -> Vec<PropertyPair<'projection>> {
    let right_by_property: HashMap<_, _> = right
        .iter()
        .map(|property| (property.decl.property, property))
        .collect();
    let left_properties: std::collections::HashSet<_> =
        left.iter().map(|property| property.decl.property).collect();

    left.iter()
        .map(|left| match right_by_property.get(&left.decl.property) {
            Some(right) => PropertyPair::Both(left, right),
            None => PropertyPair::LeftOnly(left),
        })
        .chain(
            right
                .iter()
                .filter(|right| !left_properties.contains(&right.decl.property))
                .map(PropertyPair::RightOnly),
        )
        .collect()
}

/// The version of the projection
///
/// Recorded in artifacts so that a consumer can tell whether two artifacts were
/// produced by the same representation rules.
pub const PROJECTION_VERSION: &str = "1";

/// The greatest nesting depth the projection will represent
///
/// Operational rather than semantic: the projection and the algorithms over it descend
/// recursively, so a pathologically deep input would exhaust the stack. The limit is
/// far beyond any real document — a scholarly article nests a few tens of levels at
/// most — and exceeding it returns [`CompareError::DepthExceeded`] rather than
/// aborting the process.
pub const MAX_DEPTH: usize = 128;

/// An index into a [`Projection`]'s arena of occurrences
pub type OccurrenceId = usize;

/// Whether a declared property is present on a value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Presence {
    /// The property is optional and not present
    ///
    /// Distinct from a present property holding an empty sequence, and from a present
    /// property holding a null value.
    Absent,

    /// The property is present
    Present,
}

/// One item of a property: either a structured occurrence or a scalar value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A structured occurrence, identified by its position in the projection's arena
    Structured(OccurrenceId),

    /// A scalar value
    Scalar(ScalarValue),
}

/// One projected property of an occurrence
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedProperty {
    /// What the schema declares about the property
    pub decl: PropertyDecl,

    /// Whether the property is present
    pub presence: Presence,

    /// The items of the property
    ///
    /// Empty when the property is absent. A singular property that is present has
    /// exactly one item; a repeated property that is present has one item per
    /// element, of which there may be none.
    pub items: Vec<Item>,
}

/// A structured occurrence: a concrete typed schema object at a [`NodePath`]
#[derive(Debug, Clone)]
pub struct Occurrence {
    /// The position of this occurrence in the projection's arena
    pub id: OccurrenceId,

    /// The concrete node type
    pub node_type: NodeType,

    /// The path to this occurrence from the projected root
    pub path: NodePath,

    /// The occurrence that contains this one, if any
    pub parent: Option<OccurrenceId>,

    /// The property of the parent that contains this occurrence
    pub parent_property: Option<NodeProperty>,

    /// The declared properties of this occurrence
    pub properties: Vec<ProjectedProperty>,

    /// The number of structured occurrences in this subtree, including this one
    ///
    /// Precomputed, in one pass, so that algorithms which need the size of a subtree
    /// neither recurse nor recompute it.
    pub subtree_size: i64,
}

impl Occurrence {
    /// A stable artifact reference to this occurrence
    pub(crate) fn node_ref(&self) -> NodeRef {
        NodeRef::new(self.path.clone(), self.node_type)
    }
}

/// The projected root
///
/// All `Node` variants are valid roots, including primitive, dynamic array and
/// dynamic object variants, so a root may be a scalar rather than an occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Root {
    Structured(OccurrenceId),
    Scalar(ScalarValue),
}

/// A canonical projection of one input
#[derive(Debug, Clone)]
pub struct Projection {
    /// Which of the two caller-selected inputs this projection is of
    side: Side,

    /// The arena of structured occurrences, in the order they were projected
    ///
    /// Projection is depth first and left to right, so a parent always precedes its
    /// descendants.
    occurrences: Vec<Occurrence>,

    /// The projected root
    root: Root,
}

impl Projection {
    /// Project a node
    pub fn new(node: &dyn InspectNode, side: Side) -> CompareResult<Self> {
        let mut projection = Self {
            side,
            occurrences: Vec::new(),
            root: Root::Scalar(ScalarValue::Null),
        };

        let root = match projection.project(node, NodePath::new(), None, None, 0)? {
            Item::Structured(id) => Root::Structured(id),
            Item::Scalar(value) => Root::Scalar(value),
        };
        projection.root = root;
        projection.compute_subtree_sizes();

        Ok(projection)
    }

    /// Fill in the size of every subtree
    ///
    /// Projection is depth first, so a parent always precedes its descendants and one
    /// reverse pass suffices. Being iterative, it also keeps the cost linear and adds
    /// no recursion depth.
    fn compute_subtree_sizes(&mut self) {
        for id in (0..self.occurrences.len()).rev() {
            let mut size = 1;
            for property in &self.occurrences[id].properties {
                for item in &property.items {
                    if let Item::Structured(child) = item {
                        size += self.occurrences[*child].subtree_size;
                    }
                }
            }
            self.occurrences[id].subtree_size = size;
        }
    }

    /// The node type of the projected root
    ///
    /// The selected roots always receive a root correspondence, so a scalar root needs
    /// a node type too. Every `Node` variant has one; a scalar that can only appear as
    /// a property value, such as a schema enum, does not and cannot be a root.
    pub fn root_node_type(&self) -> CompareResult<NodeType> {
        match &self.root {
            Root::Structured(id) => Ok(self.occurrence(*id)?.node_type),
            Root::Scalar(value) => value.node_type().ok_or_else(|| CompareError::Invariant {
                message: format!(
                    "The {side} root is a scalar that cannot be a node",
                    side = self.side
                ),
            }),
        }
    }

    /// Which of the two caller-selected inputs this projection is of
    pub fn side(&self) -> Side {
        self.side
    }

    /// The projected root
    pub fn root(&self) -> &Root {
        &self.root
    }

    /// A stable artifact reference to the projected root
    pub(crate) fn root_ref(&self) -> CompareResult<NodeRef> {
        Ok(match self.root() {
            Root::Structured(id) => self.occurrence(*id)?.node_ref(),
            Root::Scalar(..) => NodeRef::new(NodePath::new(), self.root_node_type()?),
        })
    }

    /// All structured occurrences, in projection order
    pub fn occurrences(&self) -> &[Occurrence] {
        &self.occurrences
    }

    /// An occurrence by id
    pub fn occurrence(&self, id: OccurrenceId) -> CompareResult<&Occurrence> {
        self.occurrences
            .get(id)
            .ok_or_else(|| CompareError::Invariant {
                message: format!(
                    "No occurrence with id {id} in the {side} projection",
                    side = self.side
                ),
            })
    }

    /// Project a value, returning the item that represents it
    fn project(
        &mut self,
        node: &dyn InspectNode,
        path: NodePath,
        parent: Option<OccurrenceId>,
        parent_property: Option<NodeProperty>,
        depth: usize,
    ) -> CompareResult<Item> {
        if depth > MAX_DEPTH {
            return Err(CompareError::DepthExceeded {
                side: self.side,
                path,
                depth,
                allowed: MAX_DEPTH,
            });
        }

        // A value with a node type is a structured occurrence. Union enums report the
        // node type of their selected branch, so they add no occurrence of their own,
        // and neither do flattened `*Options` containers, which report no node type
        // and whose properties are reported by their owning type.
        if let Some(node_type) = node.node_type() {
            let id = self.occurrences.len();
            self.occurrences.push(Occurrence {
                id,
                node_type,
                path: path.clone(),
                parent,
                parent_property,
                properties: Vec::new(),
                subtree_size: 1,
            });

            let mut properties = Vec::new();
            for property in node.properties() {
                properties.push(self.project_property(
                    property.decl,
                    property.value,
                    &path,
                    id,
                    depth,
                )?);
            }
            self.occurrences[id].properties = properties;

            return Ok(Item::Structured(id));
        }

        match node.scalar() {
            Some(scalar) => Ok(Item::Scalar(self.scalar_value(scalar, &path)?)),
            None => Err(CompareError::Projection {
                side: self.side,
                path,
                message: "value is neither a structured occurrence nor a scalar".to_string(),
            }),
        }
    }

    /// Project one property of an occurrence
    fn project_property(
        &mut self,
        decl: PropertyDecl,
        value: InspectValue<'_>,
        path: &NodePath,
        parent: OccurrenceId,
        depth: usize,
    ) -> CompareResult<ProjectedProperty> {
        let mut property_path = path.clone();
        property_path.push_back(NodeSlot::Property(decl.property));

        let (presence, items) = match value {
            InspectValue::Absent => (Presence::Absent, Vec::new()),
            InspectValue::One(node) => {
                let item = self.project(
                    node,
                    property_path,
                    Some(parent),
                    Some(decl.property),
                    depth + 1,
                )?;
                (Presence::Present, vec![item])
            }
            InspectValue::Many(nodes) => {
                let mut items = Vec::with_capacity(nodes.len());
                for (index, node) in nodes.into_iter().enumerate() {
                    let mut item_path = property_path.clone();
                    item_path.push_back(NodeSlot::Index(index));
                    items.push(self.project(
                        node,
                        item_path,
                        Some(parent),
                        Some(decl.property),
                        depth + 1,
                    )?);
                }
                (Presence::Present, items)
            }
        };

        Ok(ProjectedProperty {
            decl,
            presence,
            items,
        })
    }

    /// Convert a borrowed scalar into a canonical scalar value
    fn scalar_value(&self, scalar: ScalarRef<'_>, path: &NodePath) -> CompareResult<ScalarValue> {
        Ok(match scalar {
            ScalarRef::Null => ScalarValue::Null,
            ScalarRef::Boolean(value) => ScalarValue::Boolean { value },
            ScalarRef::Integer(value) => ScalarValue::Integer { value },
            ScalarRef::UnsignedInteger(value) => ScalarValue::UnsignedInteger { value },
            ScalarRef::Number(value) => ScalarValue::number(value),
            ScalarRef::String(value) => ScalarValue::string(value),
            // The intrinsic `Cord` adapter: string only, authorship ignored
            ScalarRef::Cord(cord) => ScalarValue::string(cord.string.as_str()),
            ScalarRef::Enum {
                schema_type,
                variant,
            } => ScalarValue::Enum {
                schema_type: schema_type.to_string(),
                variant: variant.to_string(),
            },
            ScalarRef::Array(array) => {
                let mut items = Vec::with_capacity(array.len());
                for primitive in array.iter() {
                    items.push(self.primitive_value(primitive, path)?);
                }
                ScalarValue::Array { items }
            }
            ScalarRef::Object(object) => {
                let mut entries = Vec::with_capacity(object.len());
                for (key, primitive) in object.iter() {
                    entries.push((key.clone(), self.primitive_value(primitive, path)?));
                }
                ScalarValue::object(entries).map_err(|error| CompareError::Scalar {
                    side: self.side,
                    path: path.clone(),
                    message: error.to_string(),
                })?
            }
        })
    }

    /// Convert a value nested within a dynamic array or object
    fn primitive_value(
        &self,
        primitive: &dyn InspectNode,
        path: &NodePath,
    ) -> CompareResult<ScalarValue> {
        match primitive.scalar() {
            Some(scalar) => self.scalar_value(scalar, path),
            None => Err(CompareError::Scalar {
                side: self.side,
                path: path.clone(),
                message: "value nested in a dynamic array or object is not a scalar".to_string(),
            }),
        }
    }

    /// Whether two projections are exactly equal
    ///
    /// This is the definition of equality for comparison: two nodes are equal exactly
    /// when their canonical projections are equal. Because paths are derived from
    /// structure, comparing structure is sufficient.
    ///
    /// Returns an error, rather than reporting inequality, if a projection is
    /// internally inconsistent: an invariant failure must never be answered as a
    /// difference.
    pub fn eq_canonically(&self, other: &Projection) -> CompareResult<bool> {
        match (&self.root, &other.root) {
            (Root::Scalar(left), Root::Scalar(right)) => Ok(left == right),
            (Root::Structured(left), Root::Structured(right)) => {
                self.eq_occurrence(*left, other, *right)
            }
            _ => Ok(false),
        }
    }

    /// Whether two projected subtrees are exactly equal
    ///
    /// Fingerprints are accelerators, never equality proofs, so a fingerprint match
    /// must always be settled by this before it is treated as an exact identity.
    pub fn eq_subtrees(
        &self,
        left: OccurrenceId,
        other: &Projection,
        right: OccurrenceId,
    ) -> CompareResult<bool> {
        self.eq_occurrence_with(left, other, right, Identity::Included)
    }

    /// Whether two projected subtrees are equal apart from their explicit `id`s
    ///
    /// Identity-neutral equality is what makes an otherwise exact move recognisable
    /// even when its `id` was edited along the way.
    pub fn eq_subtrees_identity_neutral(
        &self,
        left: OccurrenceId,
        other: &Projection,
        right: OccurrenceId,
    ) -> CompareResult<bool> {
        self.eq_occurrence_with(left, other, right, Identity::Neutral)
    }

    fn eq_occurrence(
        &self,
        left: OccurrenceId,
        other: &Projection,
        right: OccurrenceId,
    ) -> CompareResult<bool> {
        self.eq_occurrence_with(left, other, right, Identity::Included)
    }

    fn eq_occurrence_with(
        &self,
        left: OccurrenceId,
        other: &Projection,
        right: OccurrenceId,
        identity: Identity,
    ) -> CompareResult<bool> {
        let left = self.occurrence(left)?;
        let right = other.occurrence(right)?;

        if left.node_type != right.node_type || left.properties.len() != right.properties.len() {
            return Ok(false);
        }

        for (left, right) in left.properties.iter().zip(right.properties.iter()) {
            if identity == Identity::Neutral && left.decl.property == NodeProperty::Id {
                continue;
            }

            if left.decl != right.decl
                || left.presence != right.presence
                || left.items.len() != right.items.len()
            {
                return Ok(false);
            }

            for (left, right) in left.items.iter().zip(right.items.iter()) {
                let equal = match (left, right) {
                    (Item::Scalar(left), Item::Scalar(right)) => left == right,
                    (Item::Structured(left), Item::Structured(right)) => {
                        self.eq_occurrence_with(*left, other, *right, identity)?
                    }
                    _ => false,
                };
                if !equal {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
#[path = "projection/tests.rs"]
mod tests;
