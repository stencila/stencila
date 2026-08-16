//! Stable structural fingerprints
//!
//! Fingerprints are accelerators, never equality proofs: they narrow the candidates
//! that are worth verifying, and a collision must always be settled by comparing the
//! projected values themselves.
//!
//! The hash is FNV-1a rather than the standard library's default hasher, because the
//! default hasher's output is explicitly not stable across releases and artifacts must
//! be byte-for-byte reproducible.

use stencila_node_type::NodeProperty;
use stencila_schema::ValueKind;

use crate::{
    error::{CompareError, CompareResult},
    projection::{Item, OccurrenceId, Presence, Projection},
    scalar::ScalarValue,
};

/// An FNV-1a hasher
#[derive(Debug, Clone, Copy)]
pub struct Fingerprinter(u64);

impl Default for Fingerprinter {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Fingerprinter {
    /// Create a fingerprinter
    pub fn new() -> Self {
        Self::default()
    }

    /// Absorb bytes
    pub fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100_0000_01b3);
        }
    }

    /// Absorb an unsigned integer
    pub fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    /// Absorb a length-prefixed string, so that concatenations cannot collide
    pub fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        self.write(value.as_bytes());
    }

    /// The fingerprint absorbed so far
    pub fn finish(self) -> u64 {
        self.0
    }
}

/// Whether a fingerprint covers the explicit schema `id` of the occurrences it spans
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Identity {
    /// The explicit `id` is part of the fingerprint
    Included,

    /// The explicit `id` is excluded, so that editing an `id` cannot hide an otherwise
    /// exact match
    Neutral,
}

impl Identity {
    /// Whether a property is excluded under this identity
    fn excludes(self, property: NodeProperty) -> bool {
        self == Self::Neutral && property == NodeProperty::Id
    }
}

/// The structural fingerprint of a projected subtree
///
/// Combines the fingerprints already computed for the occurrence's structured
/// children, in the manner of a Merkle tree, so that fingerprinting a whole projection
/// is linear in its size rather than quadratic. Because a projection lists parents
/// before their descendants, computing in reverse order guarantees that a child's
/// fingerprint is available before its parent needs it.
///
/// `computed` must be indexed by occurrence id and already hold the fingerprints of
/// every structured child of `id`.
pub fn subtree(
    projection: &Projection,
    id: OccurrenceId,
    identity: Identity,
    computed: &[u64],
) -> CompareResult<u64> {
    let occurrence = projection.occurrence(id)?;

    let mut fingerprinter = Fingerprinter::new();
    fingerprinter.write_str(&occurrence.node_type.to_string());
    for property in &occurrence.properties {
        if identity.excludes(property.decl.property) {
            continue;
        }

        fingerprinter.write_str(&property.decl.property.to_string());
        absorb_presence(&mut fingerprinter, property.presence);
        fingerprinter.write_u64(property.items.len() as u64);
        for item in &property.items {
            match item {
                Item::Scalar(value) => {
                    fingerprinter.write_u64(0);
                    absorb_scalar(&mut fingerprinter, value);
                }
                Item::Structured(child) => {
                    fingerprinter.write_u64(1);
                    let Some(child) = computed.get(child.index()) else {
                        return Err(CompareError::Invariant {
                            message: format!(
                                "The fingerprint of the child of the occurrence at `{path}` \
                                 has not been computed",
                                path = occurrence.path
                            ),
                        });
                    };
                    fingerprinter.write_u64(*child);
                }
            }
        }
    }

    Ok(fingerprinter.finish())
}

/// The fingerprint of an occurrence's own non-textual scalar properties
///
/// A shallow signature distinguishes occurrences that differ in their own values but
/// whose descendants are similar — a heading's level, a citation's mode, a code
/// chunk's language.
///
/// Strings are excluded, because every string an occurrence declares already feeds the
/// separate text-similarity signal, and counting it twice would make a rewritten leaf
/// look twice as different as it is. Structured properties are excluded too: their
/// contents are the subject of the fingerprint, not of the signature.
pub fn scalar_signature(
    projection: &Projection,
    id: OccurrenceId,
    identity: Identity,
) -> CompareResult<u64> {
    let occurrence = projection.occurrence(id)?;

    let mut fingerprinter = Fingerprinter::new();
    fingerprinter.write_str(&occurrence.node_type.to_string());
    for property in &occurrence.properties {
        if identity.excludes(property.decl.property) {
            continue;
        }
        let scalars: Vec<&ScalarValue> = property
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Scalar(ScalarValue::String { .. }) => None,
                Item::Scalar(value) => Some(value),
                Item::Structured(..) => None,
            })
            .collect();

        // A property contributes when the schema declares it to hold scalars, so that
        // its absence counts, or when a union slot happens to hold scalar items
        if scalars.is_empty() && property.decl.kind != ValueKind::Scalar {
            continue;
        }

        fingerprinter.write_str(&property.decl.property.to_string());
        absorb_presence(&mut fingerprinter, property.presence);
        fingerprinter.write_u64(scalars.len() as u64);
        for scalar in scalars {
            absorb_scalar(&mut fingerprinter, scalar);
        }
    }

    Ok(fingerprinter.finish())
}

/// The fingerprint of a scalar value
///
/// Lets an exact scalar item of a mixed collection act as an anchor for the structure
/// around it, without becoming an occurrence.
pub fn scalar(value: &ScalarValue) -> u64 {
    let mut fingerprinter = Fingerprinter::new();
    absorb_scalar(&mut fingerprinter, value);
    fingerprinter.finish()
}

fn absorb_presence(fingerprinter: &mut Fingerprinter, presence: Presence) {
    fingerprinter.write_u64(match presence {
        Presence::Absent => 0,
        Presence::Present => 1,
    });
}

fn absorb_scalar(fingerprinter: &mut Fingerprinter, value: &ScalarValue) {
    match value {
        ScalarValue::Null => fingerprinter.write_u64(0),
        ScalarValue::Boolean { value } => {
            fingerprinter.write_u64(1);
            fingerprinter.write_u64(u64::from(*value));
        }
        ScalarValue::Integer { value } => {
            fingerprinter.write_u64(2);
            fingerprinter.write_u64(*value as u64);
        }
        ScalarValue::UnsignedInteger { value } => {
            fingerprinter.write_u64(3);
            fingerprinter.write_u64(*value);
        }
        ScalarValue::Number { value } => {
            fingerprinter.write_u64(4);
            // The canonical number has already collapsed NaN payloads and negative
            // zero, so its bits are a faithful key
            fingerprinter.write_u64(value.get().to_bits());
        }
        ScalarValue::String { value } => {
            fingerprinter.write_u64(5);
            fingerprinter.write_str(value);
        }
        ScalarValue::Enum {
            schema_type,
            variant,
        } => {
            fingerprinter.write_u64(6);
            fingerprinter.write_str(schema_type);
            fingerprinter.write_str(variant);
        }
        ScalarValue::Array { items } => {
            fingerprinter.write_u64(7);
            fingerprinter.write_u64(items.len() as u64);
            for item in items {
                absorb_scalar(fingerprinter, item);
            }
        }
        ScalarValue::Object { entries } => {
            fingerprinter.write_u64(8);
            fingerprinter.write_u64(entries.len() as u64);
            for (key, value) in entries.iter() {
                fingerprinter.write_str(key);
                absorb_scalar(fingerprinter, value);
            }
        }
    }
}
