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
    error::{CompareError, CompareResult},
    fingerprint::{self, Identity},
    person::PersonIdentity,
    projection::{Item, OccurrenceId, Presence, Projection},
    references::{TargetIndex, is_identity},
    scalar::ScalarValue,
    text::{Grams, NormalizedText},
};

/// The features of one projected occurrence
#[derive(Debug, Clone)]
pub struct Features {
    /// The concrete node type
    pub node_type: NodeType,

    /// The explicit schema `id`, when present and not empty
    pub explicit_id: Option<String>,

    /// A citation or internal link target, used only as positive matching evidence.
    /// Different local labels are not evidence of different content.
    pub reference_target: Option<String>,

    /// The n-grams of the reference that a citation resolves to on its own side
    ///
    /// Citation labels are often positional, reassigned whenever a reference is added
    /// or removed, so whether two citations cite the same work is judged by this rather
    /// than by their labels.
    pub cited: Option<Grams>,

    /// The canonical structural fingerprint of the subtree
    pub fingerprint: u64,

    /// The structural fingerprint of the subtree, excluding IDs and cross-reference targets
    ///
    /// Renaming local labels cannot hide an otherwise exact match.
    pub identity_neutral_fingerprint: u64,

    /// The shallow signature of the occurrence's own scalar properties
    pub scalar_signature: u64,

    /// The normalized text of the subtree, truncated to the matching limit
    pub text: String,

    /// The character n-grams of that text
    pub grams: Grams,

    /// The number of structured occurrences in the subtree, including this one
    pub subtree_size: i64,

    /// The identifiers and own names of a person, for every `Person` occurrence and
    /// every `AuthorRole` of a person
    pub person: Option<PersonIdentity>,
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
        let mut texts = vec![NormalizedText::default(); count];

        for index in (0..count).rev() {
            let id = OccurrenceId::new(index);
            fingerprints[index] =
                fingerprint::subtree(projection, id, Identity::Included, &fingerprints)?;
            identity_neutral[index] =
                fingerprint::subtree(projection, id, Identity::Neutral, &identity_neutral)?;
            texts[index] = subtree_text(projection, id, &texts)?;
        }

        let targets = TargetIndex::new(projection);
        let features = (0..count)
            .map(|index| {
                let id = OccurrenceId::new(index);
                let occurrence = projection.occurrence(id)?;
                let reference_target = occurrence.properties.iter().find_map(|property| {
                    if property.decl.property == NodeProperty::Target
                        && is_identity(occurrence.node_type, property)
                        && let Some(Item::Scalar(ScalarValue::String { value })) =
                            property.items.first()
                        && !value.is_empty()
                    {
                        Some(value.clone())
                    } else {
                        None
                    }
                });
                let cited = match (occurrence.node_type, &reference_target) {
                    (NodeType::Citation, Some(target)) => targets
                        .resolve(
                            NodeType::Citation,
                            &ScalarValue::String {
                                value: target.clone(),
                            },
                        )
                        .and_then(|reference| texts.get(reference.index()))
                        .filter(|text| !text.as_str().is_empty())
                        .map(|text| Grams::new(text.as_str())),
                    _ => None,
                };

                Ok(Features {
                    node_type: occurrence.node_type,
                    explicit_id: explicit_id(projection, id)?,
                    reference_target,
                    cited,
                    fingerprint: fingerprints[index],
                    identity_neutral_fingerprint: identity_neutral[index],
                    scalar_signature: fingerprint::scalar_signature(
                        projection,
                        id,
                        Identity::Neutral,
                    )?,
                    text: texts[index].as_str().to_string(),
                    grams: Grams::new(texts[index].as_str()),
                    subtree_size: occurrence.subtree_size,
                    person: PersonIdentity::new(projection, id)?,
                })
            })
            .collect::<CompareResult<Vec<_>>>()?;

        Ok(Self { features })
    }

    /// The features of an occurrence
    pub fn get(&self, id: OccurrenceId) -> CompareResult<&Features> {
        self.features
            .get(id.index())
            .ok_or_else(|| CompareError::Invariant {
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

/// The matching text that stands in for any cross-reference target
///
/// Long enough to form n-grams, so a citation compared with its rendered self shares
/// most of them, and bracketed so that it is unlikely to coincide with prose.
const TARGET_PLACEHOLDER: &str = "⟦target⟧";

/// The text of an occurrence and its descendants, before normalization
///
/// Every string scalar contributes except IDs and cross-reference targets, which
/// identify nodes rather than describe their content. A cross-reference still
/// contributes a fixed placeholder whatever its label, so that content consisting only
/// of a citation stays comparable with the same content once the citation is rendered.
fn subtree_text(
    projection: &Projection,
    id: OccurrenceId,
    computed: &[NormalizedText],
) -> CompareResult<NormalizedText> {
    let mut text = NormalizedText::default();

    let occurrence = projection.occurrence(id)?;
    for property in &occurrence.properties {
        if property.presence == Presence::Absent {
            continue;
        }
        if is_identity(occurrence.node_type, property) {
            if property.decl.property == NodeProperty::Target {
                text.append(TARGET_PLACEHOLDER);
            }
            continue;
        }

        for item in &property.items {
            match item {
                Item::Scalar(ScalarValue::String { value }) => {
                    text.append(value);
                }
                Item::Scalar(..) => {}
                Item::Structured(child) => {
                    let Some(child) = computed.get(child.index()) else {
                        return Err(CompareError::Invariant {
                            message: format!(
                                "The text of the occurrence with id {child}, a child of the occurrence with id {id}, has not been computed"
                            ),
                        });
                    };
                    text.append(child.as_str());
                }
            }
        }
    }

    Ok(text)
}
