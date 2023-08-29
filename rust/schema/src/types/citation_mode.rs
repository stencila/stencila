// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The mode of a `Cite`.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml, ToText)]
#[serde(crate = "common::serde")]
pub enum CitationMode {
    #[default]
    Parenthetical,
    Narrative,
    NarrativeAuthor,
}
