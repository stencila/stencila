//! Alignment-independent features of projected occurrences
//!
//! Features are computed once per occurrence, before any correspondence is proposed,
//! and never depend on a proposed correspondence. In particular, descendants are never
//! trial-aligned in order to score a parent candidate: that is circular, costs
//! substantially more, and produces evidence that cannot be explained. Descendant-derived
//! text and fingerprints are acceptable precisely because they are computed
//! independently of any proposal.

use stencila_node_type::{NodeProperty, NodeType};

use crate::{
    error::CompareResult,
    fingerprint::{self, Identity},
    projection::{Item, OccurrenceId, Presence, Projection},
    scalar::ScalarValue,
    text::{self, Grams},
};

/// The features of one projected occurrence
#[derive(Debug, Clone)]
#[allow(dead_code, reason = "read by reconciliation and difference derivation")]
pub struct Features {
    /// The concrete node type
    pub node_type: NodeType,

    /// The explicit schema `id`, when present and not empty
    pub explicit_id: Option<String>,

    /// The canonical structural fingerprint of the subtree
    pub fingerprint: u64,

    /// The structural fingerprint of the subtree, excluding explicit `id`
    ///
    /// Excluding `id` means that editing an `id` cannot hide an otherwise exact match.
    pub identity_neutral_fingerprint: u64,

    /// The shallow signature of the occurrence's own scalar properties
    pub scalar_signature: u64,

    /// The normalized text extracted from the occurrence and its descendants
    pub text: String,

    /// The character n-grams of that text
    pub grams: Grams,

    /// The number of structured occurrences in the subtree, including this one
    pub subtree_size: i64,

    /// The occurrence that contains this one
    pub parent: Option<OccurrenceId>,

    /// The property of the parent that contains this occurrence
    pub parent_property: Option<NodeProperty>,

    /// The position of this occurrence within a repeated property of its parent
    pub position: Option<usize>,
}

/// The features of every occurrence of a projection, indexed by occurrence id
#[derive(Debug, Clone)]
pub struct FeatureSet {
    features: Vec<Features>,
}

impl FeatureSet {
    /// Compute the features of every occurrence of a projection
    ///
    /// Occurrences are visited in reverse projection order, so that the features of an
    /// occurrence's descendants are available before its own are computed. The whole
    /// pass is linear in the size of the projection.
    pub fn new(projection: &Projection) -> CompareResult<Self> {
        let count = projection.occurrences().len();

        let mut fingerprints = vec![0u64; count];
        let mut identity_neutral = vec![0u64; count];
        let mut texts: Vec<String> = vec![String::new(); count];

        for id in (0..count).rev() {
            fingerprints[id] =
                fingerprint::subtree(projection, id, Identity::Included, &fingerprints)?;
            identity_neutral[id] =
                fingerprint::subtree(projection, id, Identity::Neutral, &identity_neutral)?;
            texts[id] = subtree_text(projection, id, &texts)?;
        }

        let mut features = Vec::with_capacity(count);
        for id in 0..count {
            let occurrence = projection.occurrence(id)?;
            let text = text::normalize(&texts[id]);
            let grams = Grams::new(&text);

            features.push(Features {
                node_type: occurrence.node_type,
                explicit_id: explicit_id(projection, id)?,
                fingerprint: fingerprints[id],
                identity_neutral_fingerprint: identity_neutral[id],
                scalar_signature: fingerprint::scalar_signature(
                    projection,
                    id,
                    Identity::Included,
                )?,
                text,
                grams,
                subtree_size: occurrence.subtree_size,
                parent: occurrence.parent,
                parent_property: occurrence.parent_property,
                position: occurrence.parent_index,
            });
        }

        Ok(Self { features })
    }

    /// The features of an occurrence
    pub fn get(&self, id: OccurrenceId) -> CompareResult<&Features> {
        self.features
            .get(id)
            .ok_or_else(|| crate::error::CompareError::Invariant {
                message: format!("No features for the occurrence with id {id}"),
            })
    }
}

/// The explicit schema `id` of an occurrence, when present and not empty
///
/// An empty `id` is treated as no `id` at all, so that it cannot act as an identity
/// anchor shared by every occurrence that happens to carry one.
fn explicit_id(projection: &Projection, id: OccurrenceId) -> CompareResult<Option<String>> {
    for property in &projection.occurrence(id)?.properties {
        if property.decl.property != NodeProperty::Id {
            continue;
        }
        if let Some(Item::Scalar(ScalarValue::String { value })) = property.items.first()
            && !value.is_empty()
        {
            return Ok(Some(value.clone()));
        }
    }

    Ok(None)
}

/// The text of an occurrence and its descendants, before normalization
///
/// Every string scalar the occurrence declares contributes, except its explicit `id`,
/// which is a separate identity signal rather than content. Which properties are text
/// is a schema fact, so no property names are interpreted here.
fn subtree_text(
    projection: &Projection,
    id: OccurrenceId,
    computed: &[String],
) -> CompareResult<String> {
    let mut text = String::new();

    for property in &projection.occurrence(id)?.properties {
        if property.decl.property == NodeProperty::Id || property.presence == Presence::Absent {
            continue;
        }

        for item in &property.items {
            match item {
                Item::Scalar(ScalarValue::String { value }) => {
                    text.push_str(value);
                    text.push(' ');
                }
                Item::Scalar(..) => {}
                Item::Structured(child) => {
                    if let Some(child) = computed.get(*child) {
                        text.push_str(child);
                        text.push(' ');
                    }
                }
            }
        }
    }

    Ok(text)
}
