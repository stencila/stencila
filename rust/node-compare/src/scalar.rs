//! The typed scalar model used by the canonical projection
//!
//! Scalars are tagged rather than untyped JSON values, so that scalar type and
//! union-variant identity are preserved, absence is distinguished from null, and
//! dynamic `Array` and `Object` values survive projection.

use std::{
    cmp::Ordering,
    fmt::{self, Display},
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

use stencila_node_type::NodeType;

/// A canonical floating point number
///
/// Ordinary IEEE `f64` equality is neither reflexive (`NaN != NaN`) nor able to
/// distinguish values that a document author would consider different. The canonical
/// treatment is:
///
/// - every NaN payload normalizes to one reflexive `NaN` value;
/// - positive and negative zero compare equal, consistent with ordinary numeric
///   equality;
/// - infinity retains its sign;
/// - every other finite value is retained exactly.
#[derive(Debug, Clone, Copy)]
pub struct CanonicalNumber(f64);

impl CanonicalNumber {
    /// Create a canonical number from an `f64`
    pub fn new(value: f64) -> Self {
        if value.is_nan() {
            Self(f64::NAN)
        } else if value == 0. {
            // Collapses negative zero onto positive zero
            Self(0.)
        } else {
            Self(value)
        }
    }

    /// The value as an `f64`
    pub fn get(&self) -> f64 {
        self.0
    }

    /// The bits used for ordering and hashing
    ///
    /// NaN sorts after every other value, so that ordering is total.
    fn ordering_key(&self) -> (u8, f64) {
        if self.0.is_nan() {
            (1, 0.)
        } else {
            (0, self.0)
        }
    }
}

impl From<f64> for CanonicalNumber {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl PartialEq for CanonicalNumber {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_nan() {
            other.0.is_nan()
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for CanonicalNumber {}

impl PartialOrd for CanonicalNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CanonicalNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        let (self_nan, self_value) = self.ordering_key();
        let (other_nan, other_value) = other.ordering_key();
        self_nan.cmp(&other_nan).then_with(|| {
            self_value
                .partial_cmp(&other_value)
                .unwrap_or(Ordering::Equal)
        })
    }
}

impl Hash for CanonicalNumber {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.is_nan() {
            // One hash for the single canonical NaN
            u64::MAX.hash(state);
        } else {
            self.0.to_bits().hash(state);
        }
    }
}

impl Display for CanonicalNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.non_finite() {
            Some(token) => f.write_str(token),
            None => Display::fmt(&self.0, f),
        }
    }
}

impl CanonicalNumber {
    /// The explicit token used to serialize a non-finite value
    fn non_finite(&self) -> Option<&'static str> {
        if self.0.is_nan() {
            Some("NaN")
        } else if self.0 == f64::INFINITY {
            Some("Infinity")
        } else if self.0 == f64::NEG_INFINITY {
            Some("-Infinity")
        } else {
            None
        }
    }
}

impl Serialize for CanonicalNumber {
    /// Serialize as a JSON number, or, because JSON has no representation for them,
    /// as an explicit string token for non-finite values
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.non_finite() {
            Some(token) => serializer.serialize_str(token),
            None => serializer.serialize_f64(self.0),
        }
    }
}

impl<'de> Deserialize<'de> for CanonicalNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Number(f64),
            Token(String),
        }

        Ok(match Repr::deserialize(deserializer)? {
            Repr::Number(value) => Self::new(value),
            Repr::Token(token) => match token.as_str() {
                "NaN" => Self::new(f64::NAN),
                "Infinity" => Self::new(f64::INFINITY),
                "-Infinity" => Self::new(f64::NEG_INFINITY),
                _ => {
                    return Err(de::Error::custom(format!(
                        "Expected a number, or one of `NaN`, `Infinity` or `-Infinity`, got `{token}`"
                    )));
                }
            },
        })
    }
}

/// A canonical scalar value
///
/// Ordering of the variants is the canonical ordering of scalars of different types,
/// used when sorting difference records.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ScalarValue {
    /// A null value
    Null,

    /// A boolean
    Boolean { value: bool },

    /// A signed integer
    Integer { value: i64 },

    /// An unsigned integer
    UnsignedInteger { value: u64 },

    /// A floating point number
    Number { value: CanonicalNumber },

    /// A string
    ///
    /// A `Cord` projects to a string: only its `string` is compared, and its
    /// authorship is ignored entirely, matching the custom `PartialEq` for `Cord`.
    String { value: String },

    /// A variant of a schema enum
    Enum {
        schema_type: String,
        variant: String,
    },

    /// A dynamic array of primitives
    Array { items: Vec<ScalarValue> },

    /// A dynamic object of primitives
    ///
    /// Entries are canonically ordered by key, so that insertion order affects
    /// neither comparison nor serialization. Dynamic objects cannot contain
    /// duplicate keys.
    ///
    /// Build one with [`ScalarValue::object`], which establishes that order;
    /// deserialization re-establishes it, so canonical ordering holds in memory as
    /// well as after deserialization.
    Object { entries: ObjectEntries },
}

/// An attempt to construct a dynamic object with the same key more than once
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Dynamic object key `{key}` occurs more than once")]
pub struct DuplicateObjectKeyError {
    key: String,
}

impl DuplicateObjectKeyError {
    /// The key that occurred more than once
    pub fn key(&self) -> &str {
        &self.key
    }
}

/// Canonically ordered, unique entries of a dynamic object
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ObjectEntries(Vec<(String, ScalarValue)>);

impl ObjectEntries {
    fn try_new<I>(entries: I) -> Result<Self, DuplicateObjectKeyError>
    where
        I: IntoIterator<Item = (String, ScalarValue)>,
    {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_by(|(left, ..), (right, ..)| left.cmp(right));
        for adjacent in entries.windows(2) {
            if adjacent[0].0 == adjacent[1].0 {
                return Err(DuplicateObjectKeyError {
                    key: adjacent[0].0.clone(),
                });
            }
        }
        Ok(Self(entries))
    }

    /// Iterate over the entries in canonical key order
    pub fn iter(&self) -> impl Iterator<Item = &(String, ScalarValue)> {
        self.0.iter()
    }

    /// The number of entries
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no entries
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'de> Deserialize<'de> for ObjectEntries {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = Vec::<(String, ScalarValue)>::deserialize(deserializer)?;
        Self::try_new(entries).map_err(de::Error::custom)
    }
}

impl ScalarValue {
    /// The node type of this scalar, when it is one a `Node` can be
    ///
    /// Every primitive `Node` variant has a node type. A schema enum does not: it can
    /// only appear as a property value.
    pub fn node_type(&self) -> Option<NodeType> {
        Some(match self {
            Self::Null => NodeType::Null,
            Self::Boolean { .. } => NodeType::Boolean,
            Self::Integer { .. } => NodeType::Integer,
            Self::UnsignedInteger { .. } => NodeType::UnsignedInteger,
            Self::Number { .. } => NodeType::Number,
            Self::String { .. } => NodeType::String,
            Self::Array { .. } => NodeType::Array,
            Self::Object { .. } => NodeType::Object,
            Self::Enum { .. } => return None,
        })
    }

    /// Create a string scalar
    pub fn string<S: Into<String>>(value: S) -> Self {
        Self::String {
            value: value.into(),
        }
    }

    /// Create a number scalar
    pub fn number(value: f64) -> Self {
        Self::Number {
            value: CanonicalNumber::new(value),
        }
    }

    /// Create an object scalar, canonicalizing the order of its entries
    pub fn object<I>(entries: I) -> Result<Self, DuplicateObjectKeyError>
    where
        I: IntoIterator<Item = (String, ScalarValue)>,
    {
        Ok(Self::Object {
            entries: ObjectEntries::try_new(entries)?,
        })
    }
}
