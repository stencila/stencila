//! The built-in schema-native policies
//!
//! Alignment decisions and value equality are kept separate:
//!
//! - the alignment policy decides candidate compatibility, feature selection, pair and
//!   gap costs, collection mode, and hard rejection;
//! - the value policy decides projection exclusions, scalar adapters, and scalar
//!   equivalence.
//!
//! Matching normalization and equality normalization are separate concerns: the
//! aligner may normalize text in order to locate a pair, while the value policy still
//! records the original strings as different. Normalization used to find candidates
//! must never suppress a value difference.
//!
//! The first release has one built-in schema-native implementation of each, and no
//! serialized policy language.

use crate::{
    alignment::AlignmentCost,
    error::CompareResult,
    projection::{Item, OccurrenceId, Projection},
};

/// The name of the alignment algorithm
pub const ALGORITHM_NAME: &str = "stencila-schema-native";

/// The version of the alignment algorithm
pub const ALGORITHM_VERSION: &str = "1";

/// The name of the built-in policy
pub const POLICY_NAME: &str = "schema-native";

/// The cost of leaving an occurrence, and everything below it, unmatched
///
/// One unit per structured occurrence in the subtree, so that dropping a large subtree
/// is more costly than dropping a small one, and so that the cost of a gap is
/// independent of anything on the other side. Being a pure function of one projection,
/// it is trivially symmetric under swapping the inputs.
pub fn gap_cost(projection: &Projection, id: OccurrenceId) -> CompareResult<AlignmentCost> {
    Ok(AlignmentCost::ONE.saturating_mul(projection.occurrence(id)?.subtree_size))
}

/// The structured occurrences directly contained by an occurrence, in projection order
pub fn children(projection: &Projection, id: OccurrenceId) -> CompareResult<Vec<OccurrenceId>> {
    Ok(projection
        .occurrence(id)?
        .properties
        .iter()
        .flat_map(|property| property.items.iter())
        .filter_map(|item| match item {
            Item::Structured(id) => Some(*id),
            Item::Scalar(..) => None,
        })
        .collect())
}
