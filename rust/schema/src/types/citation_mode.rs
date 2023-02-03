//! Generated file, do not edit

use crate::prelude::*;

/// The mode of a `Cite`.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(untagged, crate = "common::serde")]
#[def = "Parenthetical"]
pub enum CitationMode {
    Parenthetical,
    Narrative,
    NarrativeAuthor,
}
