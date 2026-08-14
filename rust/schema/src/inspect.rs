//! Read-only introspection of what the Stencila Schema declares about a node
//!
//! This is a narrow, generated, read-only seam: it answers what the schema says
//! about a value, and nothing else. It contains no comparison costs, matching rules,
//! difference types, or any other comparison policy.
//!
//! It differs from [`crate::walk`] in that it visits *every* declared property, not
//! only those marked `#[walk]`, and it exposes the schema's own structure rather than
//! the Rust structure that represents it:
//!
//! - every value with a [`NodeType`] is a structured occurrence;
//! - generated union enums such as [`Node`], [`Block`] and [`Inline`] are transparent
//!   wrappers around their selected branch;
//! - generated `*Options` structs are flattened property containers, so their
//!   properties are reported as properties of their owning type;
//! - scalar wrappers and dynamic primitive values remain values.
//!
//! The traits live in this crate, rather than in a crate of their own beneath it,
//! because they borrow schema types: [`Cord`], [`Array`], [`Object`] and [`Null`] all
//! appear in [`ScalarRef`], so a separate trait crate would have to depend on this one
//! and the dependency would be circular. This follows [`crate::walk::WalkNode`] and
//! [`crate::patch::PatchNode`], whose traits also live here alongside their sibling
//! derive crates.
//!
//! Two traits are provided because the two questions have different shapes. The
//! static, type-level question ("what does the schema declare for this type?") is
//! answered by [`InspectType`], which is not object safe. The value-level question
//! ("what does this particular value hold?") is answered by [`InspectNode`], which is
//! object safe so that property values can be handed back as `&dyn InspectNode`.
//!
//! The intrinsic implementation machinery of the Rust representation is not reported:
//! the `uid` field is omitted entirely, and the `type` discriminator is reported
//! structurally, as the [`NodeType`], rather than as a duplicate scalar property.
//! Everything else the schema declares is reported, including explicit `id`,
//! provenance, authorship, execution state and compilation messages. In particular,
//! [`Cord`] is reported whole, as [`ScalarRef::Cord`]; deciding to compare only its
//! string is a consumer's policy, not this seam's.

use stencila_node_type::{NodeProperty, NodeType};

use crate::{Array, Cord, Null, Object};

/// The kind of value that a schema property slot holds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    /// A typed schema object, which is a structured occurrence with its own
    /// [`NodeType`] and properties
    Structured,

    /// A non-structural schema value such as a string, number, boolean, schema enum,
    /// or dynamic primitive array or object
    Scalar,

    /// A heterogeneous union, whose selected branch may be structured or scalar
    Union,

    /// A flattened property container: a generated `*Options` struct
    ///
    /// This never appears as the kind of a property, because the `options` field is
    /// flattened away; it exists only as the type-level kind of the container itself.
    Flattened,
}

/// What the schema declares about one property
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyDecl {
    /// The property
    pub property: NodeProperty,

    /// Whether the schema requires the property to be present
    pub required: bool,

    /// Whether the property holds a sequence of values, rather than a single value
    pub repeated: bool,

    /// The kind of value the property holds
    pub kind: ValueKind,
}

/// One property of a value: what the schema declares, plus what the value holds
pub struct InspectProperty<'node> {
    /// What the schema declares about the property
    pub decl: PropertyDecl,

    /// The borrowed value of the property
    pub value: InspectValue<'node>,
}

/// The borrowed value of a property
pub enum InspectValue<'node> {
    /// An optional property that is not present
    ///
    /// This is distinct from a present property holding an empty sequence, and from
    /// a present property holding a null value.
    Absent,

    /// A singular property that is present
    One(&'node dyn InspectNode),

    /// A repeated property that is present, possibly with no items
    Many(Vec<&'node dyn InspectNode>),
}

/// A borrowed scalar value
///
/// Scalars are borrowed, not cloned: a consumer builds its own representation from
/// these references.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalarRef<'node> {
    Null,
    Boolean(bool),
    Integer(i64),
    UnsignedInteger(u64),
    Number(f64),
    String(&'node str),

    /// A [`Cord`], reported whole: both its string and its authorship
    Cord(&'node Cord),

    /// A variant of a schema enum
    ///
    /// `schema_type` and `variant` are the names of the generated Rust enum and of
    /// its selected variant, which are also the names the schema uses for them.
    Enum {
        schema_type: &'static str,
        variant: &'static str,
    },

    /// A dynamic array of primitives
    Array(&'node Array),

    /// A dynamic object of primitives
    Object(&'node Object),
}

/// The static, type-level view of what the schema declares for a type
pub trait InspectType {
    /// The kind of value this type is when it appears in a property slot
    const VALUE_KIND: ValueKind;

    /// The properties the schema declares for this type
    ///
    /// Empty for scalars and for union enums; for a union, the properties depend on
    /// the selected branch, so use [`InspectNode::properties`] instead.
    fn declared_properties() -> Vec<PropertyDecl> {
        Vec::new()
    }
}

/// The value-level view of what a node holds
pub trait InspectNode {
    /// The concrete node type of this value, if it is a structured occurrence
    ///
    /// Union enums report the node type of their selected branch. Scalars, schema
    /// enums, and flattened `*Options` containers report `None`.
    fn node_type(&self) -> Option<NodeType> {
        None
    }

    /// The declared properties of this value, with their borrowed values
    fn properties(&self) -> Vec<InspectProperty<'_>> {
        Vec::new()
    }

    /// This value as a scalar, if it is one
    fn scalar(&self) -> Option<ScalarRef<'_>> {
        None
    }
}

macro_rules! scalar {
    ($type:ty, $variant:ident) => {
        impl InspectType for $type {
            const VALUE_KIND: ValueKind = ValueKind::Scalar;
        }

        impl InspectNode for $type {
            fn scalar(&self) -> Option<ScalarRef<'_>> {
                Some(ScalarRef::$variant(*self))
            }
        }
    };
}

scalar!(bool, Boolean);
scalar!(i64, Integer);
scalar!(u64, UnsignedInteger);
scalar!(f64, Number);

impl InspectType for Null {
    const VALUE_KIND: ValueKind = ValueKind::Scalar;
}

impl InspectNode for Null {
    fn scalar(&self) -> Option<ScalarRef<'_>> {
        Some(ScalarRef::Null)
    }
}

impl InspectType for String {
    const VALUE_KIND: ValueKind = ValueKind::Scalar;
}

impl InspectNode for String {
    fn scalar(&self) -> Option<ScalarRef<'_>> {
        Some(ScalarRef::String(self.as_str()))
    }
}

impl InspectType for Cord {
    const VALUE_KIND: ValueKind = ValueKind::Scalar;
}

impl InspectNode for Cord {
    fn scalar(&self) -> Option<ScalarRef<'_>> {
        Some(ScalarRef::Cord(self))
    }
}

impl InspectType for Array {
    const VALUE_KIND: ValueKind = ValueKind::Scalar;
}

impl InspectNode for Array {
    fn scalar(&self) -> Option<ScalarRef<'_>> {
        Some(ScalarRef::Array(self))
    }
}

impl InspectType for Object {
    const VALUE_KIND: ValueKind = ValueKind::Scalar;
}

impl InspectNode for Object {
    fn scalar(&self) -> Option<ScalarRef<'_>> {
        Some(ScalarRef::Object(self))
    }
}

impl<T> InspectType for Box<T>
where
    T: InspectType,
{
    const VALUE_KIND: ValueKind = T::VALUE_KIND;

    fn declared_properties() -> Vec<PropertyDecl> {
        T::declared_properties()
    }
}

impl<T> InspectNode for Box<T>
where
    T: InspectNode,
{
    fn node_type(&self) -> Option<NodeType> {
        self.as_ref().node_type()
    }

    fn properties(&self) -> Vec<InspectProperty<'_>> {
        self.as_ref().properties()
    }

    fn scalar(&self) -> Option<ScalarRef<'_>> {
        self.as_ref().scalar()
    }
}
